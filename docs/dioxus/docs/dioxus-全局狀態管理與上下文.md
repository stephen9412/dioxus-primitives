# Dioxus 全局狀態管理與上下文

在複雜的應用程序中，僅依靠組件內部狀態和父子組件間的 Props 傳遞往往不夠靈活和可維護。Dioxus 提供了強大的全局狀態管理和上下文系統，使數據可以在整個應用中高效共享和訪問。

## 1. Context API 基礎

Context API 是 Dioxus 中共享數據的主要機制，它允許數據在組件樹中向下傳遞，而無需通過 Props 顯式傳遞。

### 1.1 提供上下文

使用 `provide_context` 函數將數據添加到組件的上下文中：

```rust
fn App() -> Element {
    // 創建要共享的數據
    let theme = use_signal(|| "light".to_string());
    let user_info = UserInfo {
        username: "guest".to_string(),
        role: "visitor".to_string(),
    };
    
    // 提供給上下文
    provide_context(theme);
    provide_context(user_info);
    
    rsx! {
        div { class: "app-container",
            // 子組件可以訪問上下文
            Header {}
            Content {}
            Footer {}
        }
    }
}
```

### 1.2 消費上下文

使用 `use_context` 函數從上下文中獲取數據：

```rust
fn Header() -> Element {
    // 從上下文中獲取共享數據
    let theme = use_context::<Signal<String>>();
    let user_info = use_context::<UserInfo>();
    
    rsx! {
        header { class: "theme-{theme}",
            h1 { "歡迎, {user_info.username}!" }
            
            // 主題切換按鈕
            button {
                onclick: move |_| {
                    if theme() == "light" {
                        theme.set("dark".to_string());
                    } else {
                        theme.set("light".to_string());
                    }
                },
                if theme() == "light" { "切換到深色模式" } else { "切換到淺色模式" }
            }
        }
    }
}
```

### 1.3 上下文數據類型要求

上下文數據必須實現 `Clone` trait，這樣才能安全地被多個組件共享：

```rust
#[derive(Clone)]
struct UserInfo {
    username: String,
    role: String,
}
```

## 2. 全局狀態管理

當需要在應用的任何地方訪問狀態時，Dioxus 提供了全局狀態解決方案。

### 2.1 使用 Signal::global 創建全局狀態

```rust
use dioxus::prelude::*;

// 定義全局狀態
static THEME: GlobalSignal<String> = Signal::global(|| "light".to_string());
static COUNTER: GlobalSignal<i32> = Signal::global(|| 0);

// 在任何組件中使用全局狀態
fn ThemeSwitcher() -> Element {
    rsx! {
        button {
            onclick: move |_| {
                if *THEME.read() == "light" {
                    *THEME.write() = "dark".to_string();
                } else {
                    *THEME.write() = "light".to_string();
                }
            },
            "切換主題"
        }
    }
}

fn Counter() -> Element {
    rsx! {
        div {
            p { "計數: {COUNTER}" }
            button {
                onclick: move |_| *COUNTER.write() += 1,
                "增加"
            }
        }
    }
}
```

### 2.2 全局狀態與模塊系統

可以將相關狀態組織到模塊中，提高代碼組織性：

```rust
// state/theme.rs
use dioxus::prelude::*;

// 主題相關狀態
pub static THEME: GlobalSignal<String> = Signal::global(|| "light".to_string());
pub static ACCENT_COLOR: GlobalSignal<String> = Signal::global(|| "#3498db".to_string());

// 主題管理函數
pub fn toggle_theme() {
    let mut theme = THEME.write();
    *theme = if *theme == "light" { "dark".to_string() } else { "light".to_string() };
    
    // 同時更新強調色
    let mut accent = ACCENT_COLOR.write();
    *accent = if *theme == "light" { "#3498db".to_string() } else { "#bb86fc".to_string() };
}

// state/user.rs
use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct User {
    pub id: String,
    pub name: String,
    pub is_admin: bool,
}

pub static CURRENT_USER: GlobalSignal<Option<User>> = Signal::global(|| None);

pub fn logout() {
    *CURRENT_USER.write() = None;
}

pub fn login(user: User) {
    *CURRENT_USER.write() = Some(user);
}
```

在組件中使用：

```rust
// 導入狀態模塊
use crate::state::theme::{THEME, toggle_theme};
use crate::state::user::{CURRENT_USER, login, logout};

fn UserPanel() -> Element {
    rsx! {
        div { class: "user-panel theme-{THEME}",
            if let Some(user) = &*CURRENT_USER.read() {
                rsx! {
                    p { "歡迎, {user.name}" }
                    button { onclick: move |_| logout(), "登出" }
                }
            } else {
                rsx! {
                    button { onclick: move |_| {
                        login(User {
                            id: "1".to_string(),
                            name: "用戶".to_string(),
                            is_admin: false
                        })
                    }, "登入" }
                }
            }
            
            button { onclick: move |_| toggle_theme(), "切換主題" }
        }
    }
}
```

## 3. 複雜狀態管理模式

### 3.1 狀態容器模式

對於複雜應用，可以創建狀態容器來組織相關狀態和邏輯：

```rust
// 定義一個狀態容器
struct AppState {
    counter: Signal<i32>,
    history: Signal<Vec<String>>,
}

impl AppState {
    // 創建新的狀態容器
    fn new() -> Self {
        Self {
            counter: use_signal(|| 0),
            history: use_signal(|| Vec::new()),
        }
    }
    
    // 增加計數
    fn increment(&self) {
        self.counter.set(self.counter() + 1);
        self.add_to_history("Increment");
    }
    
    // 減少計數
    fn decrement(&self) {
        self.counter.set(self.counter() - 1);
        self.add_to_history("Decrement");
    }
    
    // 添加歷史記錄
    fn add_to_history(&self, action: &str) {
        self.history.update(|h| {
            h.push(format!("{}: {} -> {}", action, self.counter() - 1, self.counter()));
            if h.len() > 10 {
                h.remove(0);
            }
        });
    }
    
    // 重置
    fn reset(&self) {
        self.counter.set(0);
        self.history.set(Vec::new());
    }
}

// 通過 Context 共享狀態容器
fn App() -> Element {
    // 創建狀態容器
    let state = AppState::new();
    
    // 提供給上下文
    provide_context(state);
    
    rsx! {
        div {
            h1 { "狀態容器示例" }
            Counter {}
            History {}
        }
    }
}

fn Counter() -> Element {
    // 獲取狀態容器
    let state = use_context::<AppState>();
    
    rsx! {
        div {
            h2 { "計數器: {state.counter}" }
            button { onclick: move |_| state.increment(), "+" }
            button { onclick: move |_| state.decrement(), "-" }
            button { onclick: move |_| state.reset(), "重置" }
        }
    }
}

fn History() -> Element {
    let state = use_context::<AppState>();
    
    rsx! {
        div {
            h2 { "歷史記錄" }
            if state.history().is_empty() {
                p { "沒有歷史記錄" }
            } else {
                ul {
                    for (i, entry) in state.history().iter().enumerate() {
                        li { key: "{i}", "{entry}" }
                    }
                }
            }
        }
    }
}
```

### 3.2 使用協程管理全局狀態

協程可以作為全局狀態管理的另一種強大方式，尤其適合需要異步操作的場景：

```rust
// 定義訊息類型
enum StoreAction {
    FetchProducts,
    AddToCart(String),
    RemoveFromCart(String),
    ClearCart,
}

// 全局狀態
static PRODUCTS: GlobalSignal<Vec<Product>> = Signal::global(Vec::new);
static CART: GlobalSignal<HashMap<String, u32>> = Signal::global(HashMap::new);
static LOADING: GlobalSignal<bool> = Signal::global(|| false);

fn App() -> Element {
    // 創建全局狀態管理協程
    let _store = use_coroutine(|mut rx: UnboundedReceiver<StoreAction>| async move {
        use futures_util::StreamExt;
        
        while let Some(action) = rx.next().await {
            match action {
                StoreAction::FetchProducts => {
                    *LOADING.write() = true;
                    
                    match fetch_products().await {
                        Ok(products) => {
                            *PRODUCTS.write() = products;
                        }
                        Err(err) => {
                            println!("獲取產品失敗: {}", err);
                        }
                    }
                    
                    *LOADING.write() = false;
                }
                
                StoreAction::AddToCart(product_id) => {
                    let mut cart = CART.write();
                    *cart.entry(product_id).or_insert(0) += 1;
                }
                
                StoreAction::RemoveFromCart(product_id) => {
                    let mut cart = CART.write();
                    if let Some(count) = cart.get_mut(&product_id) {
                        *count -= 1;
                        if *count == 0 {
                            cart.remove(&product_id);
                        }
                    }
                }
                
                StoreAction::ClearCart => {
                    *CART.write() = HashMap::new();
                }
            }
        }
    });
    
    // 初始加載產品
    use_effect(move |_| {
        let store_handle = use_coroutine_handle::<StoreAction>();
        store_handle.send(StoreAction::FetchProducts);
        None
    });
    
    rsx! {
        div {
            h1 { "購物應用" }
            ProductList {}
            Cart {}
        }
    }
}

fn ProductList() -> Element {
    let store_handle = use_coroutine_handle::<StoreAction>();
    
    rsx! {
        div {
            h2 { "產品列表" }
            
            if *LOADING.read() {
                p { "加載中..." }
            } else if PRODUCTS.read().is_empty() {
                p { "沒有可用產品" }
                button {
                    onclick: move |_| store_handle.send(StoreAction::FetchProducts),
                    "刷新"
                }
            } else {
                ul {
                    for product in PRODUCTS.read().iter() {
                        li {
                            "{product.name} - {product.price}元"
                            button {
                                onclick: move |_| store_handle.send(
                                    StoreAction::AddToCart(product.id.clone())
                                ),
                                "加入購物車"
                            }
                        }
                    }
                }
            }
        }
    }
}

fn Cart() -> Element {
    let store_handle = use_coroutine_handle::<StoreAction>();
    let cart = CART.read();
    let products = PRODUCTS.read();
    
    // 計算總價
    let total = cart.iter().fold(0.0, |sum, (id, count)| {
        sum + products.iter()
            .find(|p| p.id == *id)
            .map(|p| p.price * (*count as f64))
            .unwrap_or(0.0)
    });
    
    rsx! {
        div {
            h2 { "購物車" }
            
            if cart.is_empty() {
                p { "購物車為空" }
            } else {
                ul {
                    for (id, count) in cart.iter() {
                        if let Some(product) = products.iter().find(|p| p.id == *id) {
                            li {
                                "{product.name} x {count} = {product.price * (*count as f64)}元"
                                button {
                                    onclick: move |_| store_handle.send(
                                        StoreAction::RemoveFromCart(id.clone())
                                    ),
                                    "減少"
                                }
                            }
                        }
                    }
                }
                
                p { "總計: {total}元" }
                button {
                    onclick: move |_| store_handle.send(StoreAction::ClearCart),
                    "清空購物車"
                }
            }
        }
    }
}

// 產品模型
#[derive(Clone, Debug)]
struct Product {
    id: String,
    name: String,
    price: f64,
}

// 模擬API調用
async fn fetch_products() -> Result<Vec<Product>, String> {
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    Ok(vec![
        Product { id: "p1".into(), name: "智能手機".into(), price: 999.0 },
        Product { id: "p2".into(), name: "筆記本電腦".into(), price: 1299.0 },
        Product { id: "p3".into(), name: "無線耳機".into(), price: 199.0 },
    ])
}
```

## 4. 創建可伸縮的應用狀態架構

### 4.1 分層狀態管理

對於大型應用，通常需要分層管理狀態：

1. **本地組件狀態**：使用 `use_signal` 等管理僅組件內部使用的狀態
2. **共享區域狀態**：使用 Context API 在相關組件間共享狀態
3. **全局應用狀態**：使用 `Signal::global` 管理跨整個應用的狀態

```rust
// 模塊化狀態管理結構
mod state {
    pub mod auth {
        use dioxus::prelude::*;
        // 身份驗證狀態...
    }
    
    pub mod cart {
        use dioxus::prelude::*;
        // 購物車狀態...
    }
    
    pub mod products {
        use dioxus::prelude::*;
        // 產品狀態...
    }
    
    pub mod ui {
        use dioxus::prelude::*;
        // UI狀態 (主題、側邊欄等)...
    }
}
```

### 4.2 基於上下文的狀態選擇器

為了避免過度重渲染，可以創建狀態選擇器，只提取組件所需的狀態部分：

```rust
// 狀態選擇器模式
fn use_selected_state<T, F>(selector: F) -> T
where
    F: Fn(&AppState) -> T,
    T: PartialEq + Clone + 'static,
{
    let state = use_context::<AppState>();
    use_memo(move |_| selector(&state))
}

// 使用狀態選擇器
fn UserBadge() -> Element {
    // 只訂閱用戶名，不訂閱其他狀態
    let username = use_selected_state(|state| state.user.username.clone());
    
    rsx! {
        span { class: "badge", "{username}" }
    }
}
```

### 4.3 狀態持久化

結合本地存儲實現狀態持久化：

```rust
// 持久化全局狀態
#[cfg(feature = "web")]
fn use_persisted_state<T>(key: &str, default: T) -> Signal<T>
where
    T: 'static + Clone + serde::Serialize + serde::de::DeserializeOwned,
{
    use_signal(|| {
        // 嘗試從本地存儲加載
        if let Ok(Some(stored)) = gloo_storage::LocalStorage::get(key) {
            stored
        } else {
            default
        }
    })
}

// 監聽狀態變化並持久化
#[cfg(feature = "web")]
fn persist_state<T>(key: &str, state: Signal<T>)
where
    T: 'static + Clone + serde::Serialize,
{
    // 當狀態變化時保存到本地存儲
    use_effect(move |_| {
        let state_value = state();
        let _ = gloo_storage::LocalStorage::set(key, &state_value);
        None
    });
}

// 使用持久化狀態
#[cfg(feature = "web")]
fn App() -> Element {
    // 創建持久化狀態
    let settings = use_persisted_state("app_settings", Settings::default());
    
    // 設置持久化監聽
    persist_state("app_settings", settings);
    
    // 提供給上下文
    provide_context(settings);
    
    // 渲染組件...
    todo!()
}
```

## 5. 使用 MobX 風格的響應式狀態

Dioxus 的響應式系統支持 MobX 風格的狀態管理模式，使數據變化自動觸發視圖更新：

```rust
// 定義存儲類型
struct TodoStore {
    todos: Signal<Vec<Todo>>,
}

#[derive(Clone, Debug)]
struct Todo {
    id: usize,
    text: String,
    completed: bool,
}

impl TodoStore {
    fn new() -> Self {
        Self {
            todos: use_signal(Vec::new),
        }
    }
    
    // 計算屬性
    fn completed_count(&self) -> usize {
        self.todos.read().iter().filter(|t| t.completed).count()
    }
    
    fn active_count(&self) -> usize {
        self.todos.read().len() - self.completed_count()
    }
    
    // 操作
    fn add_todo(&self, text: String) {
        let next_id = self.todos.read().len();
        self.todos.write().push(Todo {
            id: next_id,
            text,
            completed: false,
        });
    }
    
    fn toggle_todo(&self, id: usize) {
        let mut todos = self.todos.write();
        if let Some(todo) = todos.iter_mut().find(|t| t.id == id) {
            todo.completed = !todo.completed;
        }
    }
    
    fn delete_todo(&self, id: usize) {
        self.todos.update(|todos| {
            todos.retain(|t| t.id != id);
        });
    }
    
    fn clear_completed(&self) {
        self.todos.update(|todos| {
            todos.retain(|t| !t.completed);
        });
    }
}

// 在應用中使用
fn TodoApp() -> Element {
    // 創建並提供存儲
    let store = TodoStore::new();
    provide_context(store.clone());
    
    rsx! {
        div { class: "todo-app",
            TodoInput {}
            TodoList {}
            TodoFooter {}
        }
    }
}

fn TodoInput() -> Element {
    let store = use_context::<TodoStore>();
    let new_todo = use_signal(|| String::new());
    
    rsx! {
        div { class: "todo-input",
            input {
                placeholder: "添加待辦事項...",
                value: new_todo(),
                oninput: move |evt| {
                    if let Some(value) = evt.value() {
                        new_todo.set(value);
                    }
                },
                onkeydown: move |evt| {
                    if evt.key() == "Enter" && !new_todo().trim().is_empty() {
                        store.add_todo(new_todo());
                        new_todo.set(String::new());
                    }
                }
            }
        }
    }
}

fn TodoList() -> Element {
    let store = use_context::<TodoStore>();
    
    rsx! {
        ul { class: "todo-list",
            for todo in store.todos.read().iter() {
                li {
                    key: "{todo.id}",
                    class: if todo.completed { "completed" } else { "" },
                    
                    input {
                        r#type: "checkbox",
                        checked: todo.completed,
                        onchange: move |_| store.toggle_todo(todo.id)
                    }
                    
                    span { "{todo.text}" }
                    
                    button {
                        onclick: move |_| store.delete_todo(todo.id),
                        "×"
                    }
                }
            }
        }
    }
}

fn TodoFooter() -> Element {
    let store = use_context::<TodoStore>();
    
    // 無需使用 use_memo - Signal 自動追蹤依賴
    let active_count = store.active_count();
    let completed_count = store.completed_count();
    let total_count = active_count + completed_count;
    
    rsx! {
        div { class: "todo-footer",
            if total_count > 0 {
                rsx! {
                    span { "{active_count} 項待完成" }
                    
                    if completed_count > 0 {
                        button {
                            onclick: move |_| store.clear_completed(),
                            "清除已完成 ({completed_count})"
                        }
                    }
                }
            } else {
                rsx! {
                    span { "沒有待辦事項" }
                }
            }
        }
    }
}
```

## 6. 最佳實踐

### 6.1 狀態分割原則

- **粒度適中**：避免過於細碎或過於龐大的狀態
- **關注點分離**：按功能和領域劃分狀態
- **最小化共享**：僅共享必要的狀態，避免過度耦合

### 6.2 性能優化

- **避免過大的全局狀態**：大狀態可能導致更新緩慢
- **適當使用 `use_memo`**：對計算密集的數據處理進行緩存
- **細粒度選擇器**：只訂閱組件實際需要的狀態部分

### 6.3 類型安全與文檔

- **強類型設計**：利用 Rust 的類型系統確保狀態操作安全
- **清晰文檔**：為狀態操作添加明確的文檔
- **抽象統一接口**：提供一致的狀態訪問和修改模式

Dioxus 的全局狀態管理和上下文系統為構建大型應用提供了強大而靈活的工具。無論是簡單的主題切換還是複雜的電子商務數據流，都可以通過這些機制優雅地實現，同時保持 Rust 的安全性和性能優勢。
