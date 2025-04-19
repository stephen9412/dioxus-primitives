use dioxus::prelude::*;
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

pub fn create_context<T: 'static + Clone + PartialEq>(
    root_component_name: &str,
    default_context: Option<T>,
) -> (
    impl Fn(T, Element) -> Element + 'static,
    impl Fn(&str) -> T + 'static,
) {
    let root_component_name = root_component_name.to_string();

    // Provider component
    #[component]
    fn Provider<T: 'static + Clone + PartialEq>(value: T, children: Element) -> Element {
        // Create a signal for the context value
        let context_signal = use_signal(|| value.clone());

        // Memoize the context value to avoid unnecessary rerenders
        // Similar to React.useMemo with Object.values dependency
        let memoized_value = use_memo(move || context_signal.read().clone());

        // Provide the context
        use_context_provider(|| memoized_value.clone());

        // Render children
        rsx! { {children} }
    }

    // Wrapper function for the provider component
    let provider_fn = move |value: T, children: Element| -> Element {
        rsx! {
            Provider {
                value: value.clone(),
                children: children
            }
        }
    };

    // Consumer hook
    let use_ctx = move |consumer_name: &str| -> T {
        // 使用擁有的字符串而不是引用
        let context = use_context::<Option<T>>();

        match context {
            Some(ctx) => ctx,
            None => match &default_context {
                Some(default) => default.clone(),
                None => panic!(
                    "`{}` must be used within `{}`",
                    consumer_name, root_component_name
                ),
            },
        }
    };

    (provider_fn, use_ctx)
}

// 定義 Scope 類型
pub type ScopeContexts = Vec<Rc<dyn Any>>;
pub type Scope = Option<HashMap<String, ScopeContexts>>;

// ScopeHook 類型 - 使用 Box<dyn Fn> 而不是 fn 指針
pub type ScopeHook = Box<dyn Fn(Scope) -> HashMap<String, Scope>>;
pub type ScopeHookFactory = Arc<dyn Fn() -> ScopeHook>;

// Context 創建函數的返回類型
pub type ContextPair<T> = (ContextProvider<T>, ContextConsumer<T>);

pub struct ContextProvider<T: 'static + Clone + PartialEq> {
    scope_name: String,
    index: usize,
    _phantom: PhantomData<T>,
}

impl<T: 'static + Clone + PartialEq> ContextProvider<T> {
    pub fn render(&self, value: T, scope: Scope, children: Element) -> Element {
        // 提供上下文值
        use_context_provider(|| value);

        // 返回子元素
        rsx! { {children} }
    }
}

// Context Consumer 封裝
pub struct ContextConsumer<T: 'static + Clone + PartialEq> {
    scope_name: String,
    index: usize,
    default_context: Option<T>,
    root_name: String,
}

impl<T: 'static + Clone + PartialEq> ContextConsumer<T> {
    pub fn consume(&self, consumer_name: &str, scope: Scope) -> T {
        // 獲取上下文
        let context = use_context::<Option<T>>();

        match context {
            Some(ctx) => ctx,
            None => match &self.default_context {
                Some(default) => default.clone(),
                None => panic!(
                    "`{}` must be used within `{}`",
                    consumer_name, self.root_name
                ),
            },
        }
    }
}

// Context Creator
pub struct ContextCreator {
    scope_name: String,
    contexts: Rc<RefCell<Vec<Option<Rc<dyn Any>>>>>,
}

impl ContextCreator {
    pub fn create<T: 'static + Clone + PartialEq>(
        &self,
        root_name: &str,
        default_context: Option<T>,
    ) -> (ContextProvider<T>, ContextConsumer<T>) {
        let root_name = root_name.to_string();
        let mut contexts_mut = self.contexts.borrow_mut();
        let index = contexts_mut.len();

        // 存儲默認上下文
        contexts_mut.push(
            default_context
                .clone()
                .map(|ctx| Rc::new(ctx) as Rc<dyn Any>),
        );

        // 創建 Provider
        let provider = ContextProvider {
            scope_name: self.scope_name.clone(),
            index,
            _phantom: PhantomData,
        };

        // 創建 Consumer
        let consumer = ContextConsumer {
            scope_name: self.scope_name.clone(),
            index,
            default_context,
            root_name,
        };

        (provider, consumer)
    }
}

pub fn create_context_scope(
    scope_name: &str,
    deps: Vec<ScopeHookFactory>,
) -> (ContextCreator, ScopeHookFactory) {
    // 轉換為擁有的字符串
    let scope_name = scope_name.to_string();
    // 存儲默認上下文
    let contexts = Rc::new(RefCell::new(Vec::new()));

    // 創建上下文創建器
    let creator = ContextCreator {
        scope_name: scope_name.clone(),
        contexts: contexts.clone(),
    };

    // 創建 ScopeHook 工廠函數
    let scope_hook_factory: ScopeHookFactory = Arc::new(move || {
        let scope_name = scope_name.clone();
        let contexts = contexts.clone();

        Box::new(move |scope: Scope| {
            let mut result = HashMap::new();

            // 獲取上下文
            let contexts_vec = contexts
                .borrow()
                .iter()
                .map(|ctx| ctx.clone().unwrap_or_else(|| Rc::new(())))
                .collect();

            // 創建新的範圍
            let mut new_scope = scope.unwrap_or_else(|| HashMap::new());
            new_scope.insert(scope_name.clone(), contexts_vec);

            // 返回範圍
            result.insert(format!("__scope{}", scope_name), Some(new_scope));
            result
        })
    });

    // 調用 compose_context_scopes
    let composed_factory = if deps.is_empty() {
        // 如果沒有依賴，直接返回基本工廠
        scope_hook_factory
    } else {
        // 否則合併所有工廠
        let mut all_factories = vec![scope_hook_factory];
        all_factories.extend(deps);
        compose_context_scopes(all_factories)
    };

    (creator, composed_factory)
}

// Compose multiple context scopes
pub fn compose_context_scopes(factories: Vec<ScopeHookFactory>) -> ScopeHookFactory {
    if factories.len() == 1 {
        // 如果只有一個工廠，直接返回它
        return factories[0].clone();
    }

    // 否則創建一個新工廠，合併所有工廠的結果
    Arc::new(move || {
        // 獲取所有工廠生成的 hooks
        let hooks: Vec<ScopeHook> = factories.iter().map(|factory| factory()).collect();

        // 返回一個新的 hook，組合所有 hooks 的結果
        Box::new(move |scope: Scope| {
            let next_scopes = hooks.iter().fold(HashMap::new(), |acc, hook| {
                // 對每個 hook 調用並合併結果
                let scope_props = hook(scope.clone());
                acc.into_iter().chain(scope_props.into_iter()).collect()
            });

            // 從合併結果中提取基本範圍
            // 這裡簡化處理，實際情況可能需要更精確的邏輯
            if let Some(first_scope) = next_scopes.values().next() {
                HashMap::from([(format!("__scope{}", "baseScope"), first_scope.clone())])
            } else {
                HashMap::new()
            }
        })
    })
}

// Export types
pub type CreateScope = fn() -> Box<dyn Fn(Scope) -> HashMap<String, Scope>>;
