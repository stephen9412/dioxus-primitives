# Dioxus Hooks 系統與狀態管理

Hooks 是 Dioxus 中管理組件狀態和副作用的核心機制。它們使函數組件能夠使用狀態和其他 Dioxus 特性，從而創建動態且響應式的用戶界面。

## 1. Hooks 基礎概念

Hooks 是以 `use_` 開頭的特殊函數，允許在函數組件中增加狀態管理和生命週期功能。每個 Hook 在組件內部維護自己的狀態，當狀態變化時會觸發組件重新渲染。

```rust
fn Counter() -> Element {
    // 使用 Hook 創建組件內部狀態
    let mut count = use_signal(|| 0);
    
    rsx! {
        div {
            p { "當前計數: {count}" }
            button { onclick: move |_| count += 1, "增加" }
            button { onclick: move |_| count -= 1, "減少" }
        }
    }
}
```

## 2. 核心狀態管理 Hooks

### 2.1 use_signal

`use_signal` 是最基本的狀態管理 Hook，它創建一個可變的信號值：

```rust
// 創建初始值為 0 的信號
let mut count = use_signal(|| 0);

// 讀取當前值（自動追蹤依賴關係）
let current = count();

// 修改值 (支持操作符重載)
count += 1;
count -= 1;

// 或使用 set 方法
count.set(100);

// 可以使用 read/write 方法操作更複雜的類型
let mut users = use_signal(Vec::new);
users.push("新用戶");  // 簡便語法糖

// 等價於:
users.write().push("新用戶");
let first_user = users.read()[0];
```

`Signal` 類型實現了 `Copy` trait，因此可以輕鬆地在事件處理器等閉包中使用它。

### 2.3 use_memo

`use_memo` 根據依賴項緩存計算結果，只有當依賴項變化時才重新計算：

```rust
let items = use_signal(|| vec![1, 2, 3, 4, 5]);
let doubled_items = use_memo(move |_| {
    items.read().iter().map(|i| i * 2).collect::<Vec<_>>()
});
```

## 3. 副作用與生命週期 Hooks

### 3.1 use_effect

`use_effect` 用於執行有副作用的操作，並可以設置依賴項控制何時重新執行：

```rust
// 組件首次渲染時執行一次
use_effect(move |_| {
    println!("組件已掛載");
    
    // 可選的清理函數（在組件卸載或下次效果執行前調用）
    Some(Box::new(|| {
        println!("組件將卸載或依賴項變化");
    }))
});

// 當 count 變化時執行
let count = use_signal(|| 0);
use_effect(move |_| {
    println!("計數變化為: {}", count());
    None
});
```

### 3.2 use_resource

`use_resource` 用於處理異步操作，如 API 請求或數據加載：

```rust
// 創建一個資源來加載數據
let users = use_resource(|| async move {
    // 執行異步操作
    let response = reqwest::get("https://api.example.com/users")
        .await?
        .json::<Vec<User>>()
        .await?;
    
    Ok::<_, reqwest::Error>(response)
});

// 使用資源狀態
match &*users.read_unchecked() {
    Some(Ok(data)) => {
        // 顯示加載的數據
        rsx! {
            ul {
                for user in data {
                    li { "{user.name}" }
                }
            }
        }
    },
    Some(Err(_)) => rsx! { div { "加載失敗" } },
    None => rsx! { div { "正在加載..." } },
}

// 手動重新加載資源
button { onclick: move |_| users.restart(), "重新加載" }
```

### 3.3 use_coroutine

`use_coroutine` 允許管理長時間運行的異步任務，並支持雙向通信：

```rust
// 定義協程可接收的消息類型
enum ProfileAction {
    SetUsername(String),
    SetEmail(String),
}

// 創建協程來處理用戶資料更新
let profile_task = use_coroutine(|mut rx: UnboundedReceiver<ProfileAction>| async move {
    // 建立連接
    let mut client = connect_to_api().await;
    
    // 不斷處理新消息
    while let Some(action) = rx.next().await {
        match action {
            ProfileAction::SetUsername(name) => {
                client.update_username(name).await;
            },
            ProfileAction::SetEmail(email) => {
                client.update_email(email).await;
            }
        }
    }
});

// 向協程發送消息
button {
    onclick: move |_| profile_task.send(ProfileAction::SetUsername("新用戶名".to_string())),
    "更新用戶名"
}
```

## 4. 上下文管理 Hooks

### 4.1 use_context 與 provide_context

用於跨組件共享數據而無需通過 props 傳遞：

```rust
// 在父組件中提供上下文
fn App() -> Element {
    // 創建應用全局狀態
    let theme = use_signal(|| "light".to_string());
    
    // 提供上下文
    provide_context(theme);
    
    rsx! {
        div { ChildComponent {} }
    }
}

// 在子組件中獲取上下文
fn ChildComponent() -> Element {
    // 獲取上下文中的主題設置
    let theme = use_context::<Signal<String>>();
    
    rsx! {
        div {
            class: "theme-{theme}",
            "當前主題: {theme}"
        }
    }
}
```

## 5. 自定義 Hooks

可以創建自定義 Hooks 來重用狀態邏輯：

```rust
// 創建一個管理表單輸入的自定義 Hook
fn use_input(initial: impl Into<String>) -> (Signal<String>, EventHandler<FormEvent>) {
    let value = use_signal(|| initial.into());
    
    let on_input = move |evt: FormEvent| {
        if let Some(input_value) = evt.value() {
            value.set(input_value);
        }
    };
    
    (value, on_input.into())
}

// 在組件中使用自定義 Hook
fn LoginForm() -> Element {
    let (username, on_username_input) = use_input("");
    let (password, on_password_input) = use_input("");
    
    rsx! {
        form {
            div {
                label { "用戶名:" }
                input {
                    value: username(),
                    oninput: on_username_input
                }
            }
            div {
                label { "密碼:" }
                input {
                    r#type: "password",
                    value: password(),
                    oninput: on_password_input
                }
            }
        }
    }
}
```

從底層創建自定義 Hook：

```rust
fn use_custom_hook<T: 'static>(init: impl FnOnce() -> T) -> T {
    use_hook(|| {
        // 在這裡執行初始化邏輯
        let value = init();
        
        // 可以使用 schedule_update() 來觸發重新渲染
        let update = schedule_update();
        
        // 返回 Hook 值
        value
    })
}
```

## 6. Hooks 使用規則

為了確保 Hooks 正常工作，必須遵循以下規則：

1. **只在組件或其他 Hooks 中調用 Hooks**

   ```rust
   // ✅ 正確：在組件中使用 Hook
   fn Component() -> Element {
       let count = use_signal(|| 0);
       // ...
   }
   
   // ❌ 錯誤：在普通函數中使用 Hook
   fn normal_function() {
       let count = use_signal(|| 0); // 這將無法工作
   }
   ```

2. **只在組件頂層調用 Hooks**

   ```rust
   // ✅ 正確：在頂層使用 Hook
   fn Component() -> Element {
       let count = use_signal(|| 0);
       let name = use_signal(|| "用戶".to_string());
       
       // ...
   }
   
   // ❌ 錯誤：在條件語句中使用 Hook
   fn Component() -> Element {
       let authenticated = check_auth();
       
       if authenticated {
           let user = use_signal(|| get_user()); // 違反規則
       }
       
       // ...
   }
   ```

3. **不在循環中使用 Hooks**

   ```rust
   // ❌ 錯誤：在循環中使用 Hook
   fn Component() -> Element {
       let items = get_items();
       
       for item in items {
           let selected = use_signal(|| false); // 違反規則
       }
       
       // ...
   }
   
   // ✅ 正確：使用集合存儲狀態
   fn Component() -> Element {
       let items = get_items();
       let selections = use_signal(|| {
           let mut map = HashMap::new();
           for item in &items {
               map.insert(item.id, false);
           }
           map
       });
       
       // ...
   }
   ```

4. **不在普通閉包中使用 Hooks**

   ```rust
   // ❌ 錯誤：在閉包中使用 Hook
   fn Component() -> Element {
       let callback = || {
           let count = use_signal(|| 0); // 違反規則
       };
       
       // ...
   }
   ```

## 7. 最佳實踐

1. **保持 Hooks 命名一致性**：自定義 Hooks 應始終以 `use_` 開頭
2. **將相關邏輯封裝到自定義 Hooks**：提高代碼可重用性和組織性
3. **避免過度使用 `use_effect`**：優先考慮聲明式方法
4. **使用 `use_memo` 優化性能**：避免在每次渲染時進行昂貴計算
5. **設置適當的依賴項**：確保 effects 和資源只在必要時重新執行
6. **優先使用響應式數據源**：利用 Signal 的自動追蹤機制簡化代碼

Dioxus 的 Hooks 系統提供了一種強大且符合 Rust 風格的方式來管理組件狀態和副作用，使得構建複雜、響應式的用戶界面變得簡單而直觀。
