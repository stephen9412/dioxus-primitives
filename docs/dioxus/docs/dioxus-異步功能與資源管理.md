# Dioxus 異步功能與資源管理

Dioxus 提供了強大的異步功能，使開發者能夠輕鬆處理網絡請求、數據加載和後台任務等操作，而不會阻塞用戶界面。本文檔詳細介紹 Dioxus 中的異步數據處理和資源管理機制。

## 1. use_resource Hook

`use_resource` 是 Dioxus 中處理異步操作的主要 Hook，專為數據獲取和異步計算設計。

### 1.1 基本用法

```rust
let users_resource = use_resource(|| async move {
    // 發起異步請求
    let response = reqwest::get("https://api.example.com/users")
        .await?
        .json::<Vec<User>>()
        .await?;
    
    Ok::<_, reqwest::Error>(response)
});
```

`use_resource` 返回一個 `Resource<T, E>` 類型，可以讀取其當前狀態：

```rust
// 讀取資源當前狀態
match &*users_resource.read_unchecked() {
    // 資源加載成功
    Some(Ok(users)) => {
        rsx! {
            div {
                h2 { "用戶列表" }
                ul {
                    for user in users {
                        li { "{user.name} ({user.email})" }
                    }
                }
            }
        }
    },
    // 資源加載失敗
    Some(Err(err)) => rsx! {
        div { class: "error",
            "加載用戶數據失敗: {err}"
        }
    },
    // 資源正在加載中
    None => rsx! {
        div { class: "loading",
            "正在加載用戶數據..."
        }
    },
}
```

### 1.2 手動重新加載

`Resource` 提供了 `restart` 方法，允許手動重新執行異步操作：

```rust
rsx! {
    button {
        onclick: move |_| users_resource.restart(),
        "重新加載用戶數據"
    }
}
```

### 1.3 自動重新加載（依賴變化觸發）

`use_resource` 會自動追蹤閉包內使用的 Signal，當這些 Signal 變化時重新執行異步操作：

```rust
// 定義過濾條件
let filter = use_signal(|| "active".to_string());

// 資源會在 filter 變化時自動重新加載
let filtered_users = use_resource(move || async move {
    let current_filter = filter();
    
    reqwest::get(format!("https://api.example.com/users?status={}", current_filter))
        .await?
        .json::<Vec<User>>()
        .await
});

// UI 中提供過濾器選擇
rsx! {
    div {
        select {
            onchange: move |evt| {
                if let Some(value) = evt.value() {
                    filter.set(value);
                }
            },
            option { value: "active", "活躍用戶" }
            option { value: "inactive", "非活躍用戶" }
            option { value: "all", "所有用戶" }
        }
    }
}
```

### 1.4 使用 use_reactive 添加非響應式依賴

對於不是通過 Signal 自動追蹤的依賴項，可以使用 `use_reactive` 宏：

```rust
let user_id = "user_123";  // 非響應式值

let user_details = use_resource(use_reactive!(|(user_id,)| async move {
    reqwest::get(format!("https://api.example.com/users/{}", user_id))
        .await?
        .json::<UserDetails>()
        .await
}));
```

## 2. use_coroutine Hook

`use_coroutine` 提供了更高級的異步任務管理，適用於長時間運行的任務和需要雙向通信的場景。

### 2.1 基本用法

```rust
// 創建一個無限循環的後台任務
let background_task = use_coroutine(|_rx| async move {
    loop {
        // 執行某些定期任務
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
        println!("執行定期任務...");
    }
});
```

### 2.2 發送和接收消息

協程最強大的功能是能夠接收外部消息，實現雙向通信：

```rust
// 定義協程可以接收的消息類型
enum TaskMessage {
    FetchData(String),
    CancelOperation,
    UpdateConfig { timeout_secs: u64, retry_count: u32 },
}

// 建立一個可以接收消息的協程
let task_handler = use_coroutine(|mut rx: UnboundedReceiver<TaskMessage>| async move {
    use futures_util::StreamExt;
    
    // 基本配置
    let mut timeout = 30;
    let mut retries = 3;
    
    // 監聽消息流
    while let Some(msg) = rx.next().await {
        match msg {
            TaskMessage::FetchData(id) => {
                println!("正在獲取數據 ID: {}", id);
                // 執行數據獲取邏輯...
                
                // 可以使用前面設置的配置
                for attempt in 0..retries {
                    // 嘗試邏輯...
                }
            },
            TaskMessage::CancelOperation => {
                println!("操作已取消");
                // 取消邏輯...
            },
            TaskMessage::UpdateConfig { timeout_secs, retry_count } => {
                timeout = timeout_secs;
                retries = retry_count;
                println!("配置已更新: 超時 = {}秒, 重試次數 = {}", timeout, retries);
            }
        }
    }
});

// 在 UI 中發送消息給協程
rsx! {
    div {
        button {
            onclick: move |_| task_handler.send(TaskMessage::FetchData("user_123".to_string())),
            "獲取用戶數據"
        }
        button {
            onclick: move |_| task_handler.send(TaskMessage::CancelOperation),
            "取消操作"
        }
        button {
            onclick: move |_| task_handler.send(TaskMessage::UpdateConfig {
                timeout_secs: 60,
                retry_count: 5,
            }),
            "更新配置"
        }
    }
}
```

### 2.3 將數據從協程傳回 UI

協程可以通過 Signal 將數據傳回 UI 層：

```rust
fn DataFetcher() -> Element {
    // 創建要在協程中更新的狀態
    let status = use_signal(|| "就緒".to_string());
    let data = use_signal(|| Vec::<String>::new());
    
    // 創建協程處理數據獲取
    let fetcher = use_coroutine(|mut rx: UnboundedReceiver<String>| {
        // 創建 Signal 的克隆，以便在異步上下文中使用
        let status = status.clone();
        let data = data.clone();
        
        async move {
            use futures_util::StreamExt;
            
            while let Some(query) = rx.next().await {
                // 更新狀態
                status.set(format!("正在獲取: {}", query));
                
                // 執行異步操作
                match fetch_data(&query).await {
                    Ok(result) => {
                        status.set("獲取成功".to_string());
                        data.set(result);
                    },
                    Err(err) => {
                        status.set(format!("錯誤: {}", err));
                    }
                }
            }
        }
    });
    
    // 搜索輸入和結果顯示
    let search_term = use_signal(|| String::new());
    
    rsx! {
        div {
            h2 { "數據搜索" }
            
            div {
                input {
                    placeholder: "輸入搜索關鍵詞",
                    value: search_term(),
                    oninput: move |evt| {
                        if let Some(value) = evt.value() {
                            search_term.set(value);
                        }
                    }
                }
                
                button {
                    onclick: move |_| {
                        if !search_term().is_empty() {
                            fetcher.send(search_term());
                        }
                    },
                    "搜索"
                }
            }
            
            div {
                p { "狀態: {status}" }
                
                if !data().is_empty() {
                    ul {
                        for item in data().iter() {
                            li { "{item}" }
                        }
                    }
                } else {
                    p { "無數據顯示" }
                }
            }
        }
    }
}

// 模擬數據獲取函數
async fn fetch_data(query: &str) -> Result<Vec<String>, String> {
    // 模擬網絡延遲
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    
    // 返回假數據
    Ok(vec![
        format!("結果 1 for {}", query),
        format!("結果 2 for {}", query),
        format!("結果 3 for {}", query),
    ])
}
```

### 2.4 協程上下文注入

協程句柄會自動注入到 Context 系統中，可以在子組件中獲取和使用：

```rust
// 在父組件中創建協程
fn ParentComponent() -> Element {
    let _task = use_coroutine(|rx: UnboundedReceiver<TaskMessage>| async move {
        // 協程邏輯...
    });
    
    rsx! {
        ChildComponent {}
    }
}

// 在任意子組件中獲取協程句柄
fn ChildComponent() -> Element {
    // 通過 Context 獲取父組件創建的協程句柄
    let task_handle = use_coroutine_handle::<TaskMessage>();
    
    rsx! {
        button {
            onclick: move |_| task_handle.send(TaskMessage::FetchData("123".to_string())),
            "從子組件發送消息"
        }
    }
}
```

## 3. 異步操作模式與最佳實踐

### 3.1 數據加載狀態管理

處理異步數據時的常見模式：

```rust
fn DataDisplay() -> Element {
    let data = use_resource(|| async move {
        fetch_data().await
    });
    
    rsx! {
        div {
            // 清晰地處理三種可能的狀態
            match &*data.read_unchecked() {
                // 1. 加載中
                None => rsx! {
                    div { class: "loading-spinner",
                        "加載中..."
                    }
                },
                
                // 2. 加載失敗
                Some(Err(err)) => rsx! {
                    div { class: "error-container",
                        h3 { "出錯了!" }
                        p { "錯誤信息: {err}" }
                        button {
                            onclick: move |_| data.restart(),
                            "重試"
                        }
                    }
                },
                
                // 3. 加載成功
                Some(Ok(items)) => rsx! {
                    div { class: "data-container",
                        h3 { "數據加載成功!" }
                        ul {
                            for item in items {
                                li { "{item}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

### 3.2 無限滾動和分頁加載

使用協程實現無限滾動加載模式：

```rust
fn InfiniteList() -> Element {
    // 定義狀態
    let items = use_signal(|| Vec::<String>::new());
    let page = use_signal(|| 1);
    let is_loading = use_signal(|| false);
    let has_more = use_signal(|| true);
    
    // 使用協程處理數據加載
    let loader = use_coroutine(|mut rx: UnboundedReceiver<LoadAction>| {
        let items = items.clone();
        let page = page.clone();
        let is_loading = is_loading.clone();
        let has_more = has_more.clone();
        
        async move {
            use futures_util::StreamExt;
            
            while let Some(action) = rx.next().await {
                match action {
                    LoadAction::LoadMore => {
                        // 避免重複請求
                        if is_loading() || !has_more() {
                            continue;
                        }
                        
                        // 設置加載標誌
                        is_loading.set(true);
                        
                        // 加載數據
                        match load_page(page()).await {
                            Ok(new_items) => {
                                // 檢查是否還有更多數據
                                if new_items.is_empty() {
                                    has_more.set(false);
                                } else {
                                    // 添加新項目並增加頁碼
                                    items.update(|current| {
                                        current.extend(new_items);
                                    });
                                    page.set(page() + 1);
                                }
                            },
                            Err(_) => {
                                // 處理錯誤...
                            }
                        }
                        
                        // 清除加載標誌
                        is_loading.set(false);
                    },
                    
                    LoadAction::Reset => {
                        items.set(Vec::new());
                        page.set(1);
                        has_more.set(true);
                        
                        // 重新加載第一頁
                        rx.next().await;
                    }
                }
            }
        }
    });
    
    // 初次挂載時加載
    use_effect(move |_| {
        loader.send(LoadAction::LoadMore);
        None
    });
    
    rsx! {
        div {
            // 顯示已加載項目
            ul {
                for (index, item) in items().iter().enumerate() {
                    li { key: "{index}", "{item}" }
                }
            }
            
            // 加載更多按鈕或提示
            if is_loading() {
                div { class: "loading", "正在加載..." }
            } else if has_more() {
                button {
                    onclick: move |_| loader.send(LoadAction::LoadMore),
                    "加載更多"
                }
            } else {
                div { "已加載全部數據" }
            }
            
            // 重置按鈕
            button {
                onclick: move |_| loader.send(LoadAction::Reset),
                "重置列表"
            }
        }
    }
}

// 定義協程消息類型
enum LoadAction {
    LoadMore,
    Reset,
}

// 模擬分頁加載
async fn load_page(page: usize) -> Result<Vec<String>, ()> {
    // 模擬網絡延遲
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    
    // 模擬有限的數據頁數
    if page > 5 {
        return Ok(Vec::new());
    }
    
    // 返回模擬數據
    Ok((1..=10).map(|i| format!("項目 #{} (頁 {})", i, page)).collect())
}
```

### 3.3 數據依賴加載

處理數據之間有依賴關係的加載模式：

```rust
fn DependentDataLoader() -> Element {
    // 第一級數據
    let categories = use_signal(|| None::<Vec<Category>>);
    let selected_category_id = use_signal(|| None::<String>);
    
    // 第二級數據 (依賴於所選類別)
    let products = use_resource(move || async move {
        // 如果沒有選擇類別，則不加載
        let Some(category_id) = selected_category_id() else {
            return Ok(Vec::new());
        };
        
        // 加載選定類別的產品
        load_products_by_category(&category_id).await
    });
    
    // 初始加載類別
    let category_loader = use_coroutine(|_| {
        let categories = categories.clone();
        
        async move {
            match load_categories().await {
                Ok(data) => {
                    categories.set(Some(data));
                },
                Err(_) => {
                    // 處理錯誤...
                }
            }
        }
    });
    
    rsx! {
        div {
            // 類別選擇器
            div {
                h3 { "選擇類別" }
                
                // 加載狀態處理
                if let Some(cats) = &categories() {
                    select {
                        onchange: move |evt| {
                            selected_category_id.set(evt.value());
                        },
                        option { value: "", "-- 請選擇 --" }
                        for cat in cats {
                            option { value: "{cat.id}", "{cat.name}" }
                        }
                    }
                } else {
                    p { "正在加載類別..." }
                }
            }
            
            // 產品顯示 (依賴於所選類別)
            div {
                h3 { "產品列表" }
                
                if selected_category_id().is_none() {
                    p { "請先選擇一個類別" }
                } else {
                    match &*products.read_unchecked() {
                        None => rsx! { p { "加載產品中..." } },
                        Some(Err(_)) => rsx! { p { "加載產品失敗" } },
                        Some(Ok(items)) => {
                            if items.is_empty() {
                                rsx! { p { "該類別下沒有產品" } }
                            } else {
                                rsx! {
                                    ul {
                                        for item in items {
                                            li { "{item.name} - {item.price}元" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// 模型與API
struct Category {
    id: String,
    name: String,
}

struct Product {
    id: String,
    name: String,
    price: f64,
}

async fn load_categories() -> Result<Vec<Category>, ()> {
    // 模擬API調用...
    Ok(vec![
        Category { id: "1".into(), name: "電子產品".into() },
        Category { id: "2".into(), name: "服裝".into() },
        Category { id: "3".into(), name: "食品".into() },
    ])
}

async fn load_products_by_category(category_id: &str) -> Result<Vec<Product>, ()> {
    // 模擬API調用...
    match category_id {
        "1" => Ok(vec![
            Product { id: "101".into(), name: "智能手機".into(), price: 999.0 },
            Product { id: "102".into(), name: "筆記本電腦".into(), price: 1299.0 },
        ]),
        "2" => Ok(vec![
            Product { id: "201".into(), name: "夏季T恤".into(), price: 29.9 },
            Product { id: "202".into(), name: "牛仔褲".into(), price: 59.9 },
        ]),
        "3" => Ok(vec![
            Product { id: "301".into(), name: "水果禮盒".into(), price: 39.9 },
            Product { id: "302".into(), name: "巧克力".into(), price: 12.5 },
        ]),
        _ => Ok(Vec::new()),
    }
}
```

## 4. 錯誤處理與恢復策略

### 4.1 優雅的錯誤處理

```rust
fn RobustDataLoader() -> Element {
    let data = use_resource(|| async move {
        // 定義重試邏輯
        let mut retries = 0;
        const MAX_RETRIES: usize = 3;
        
        loop {
            match fetch_data().await {
                Ok(data) => return Ok(data),
                Err(err) => {
                    retries += 1;
                    
                    if retries >= MAX_RETRIES {
                        return Err(format!("嘗試 {} 次後失敗: {}", MAX_RETRIES, err));
                    }
                    
                    // 指數退避策略
                    let delay = std::time::Duration::from_millis(500 * 2u64.pow(retries as u32));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    });
    
    // 渲染數據或錯誤狀態
    // ...實現類似前面的例子
    todo!()
}
```

### 4.2 使用協程實現進度報告

```rust
fn ProgressiveDataLoader() -> Element {
    // 進度和數據狀態
    let progress = use_signal(|| 0);
    let data = use_signal(|| None::<Vec<String>>);
    let status = use_signal(|| "就緒".to_string());
    
    // 使用協程處理帶進度的數據加載
    let loader = use_coroutine(|rx: UnboundedReceiver<()>| {
        let progress = progress.clone();
        let data = data.clone();
        let status = status.clone();
        
        async move {
            status.set("正在加載...".to_string());
            progress.set(0);
            
            match load_data_with_progress(
                move |p| progress.set(p)
            ).await {
                Ok(result) => {
                    data.set(Some(result));
                    status.set("加載完成".to_string());
                },
                Err(err) => {
                    status.set(format!("錯誤: {}", err));
                }
            }
        }
    });
    
    rsx! {
        div {
            h2 { "帶進度的數據加載" }
            
            // 進度條
            div { class: "progress-bar",
                div {
                    class: "progress-fill",
                    style: "width: {progress}%",
                }
                span { "{progress}%" }
            }
            
            // 狀態顯示
            p { "狀態: {status}" }
            
            // 數據顯示
            if let Some(items) = &data() {
                ul {
                    for item in items {
                        li { "{item}" }
                    }
                }
            }
            
            // 加載按鈕
            button {
                onclick: move |_| loader.send(()),
                "開始加載"
            }
        }
    }
}

// 模擬帶進度的數據加載
async fn load_data_with_progress<F>(progress_callback: F) -> Result<Vec<String>, String>
where
    F: Fn(u32) + Send + 'static
{
    // 模擬分段加載過程
    let segments = 10;
    
    let mut result = Vec::new();
    
    for i in 1..=segments {
        // 更新進度
        progress_callback((i * 100 / segments) as u32);
        
        // 模擬某個步驟的處理
        tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        
        // 添加數據
        result.push(format!("數據段 {}", i));
    }
    
    Ok(result)
}
```

Dioxus 的異步功能和資源管理系統提供了靈活而強大的方式來處理網絡請求、數據加載和後台任務。通過 `use_resource` 和 `use_coroutine` 這兩個核心 Hook，可以優雅地管理異步操作，創建響應式且高性能的用戶界面。
