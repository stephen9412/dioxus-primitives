# Dioxus 框架概覽與核心功能

[Dioxus](https://dioxuslabs.com/) 是一個用 Rust 編寫的聲明式 UI 框架，專注於開發效率、性能和跨平台能力。本文檔提供 Dioxus 核心功能的概覽，並引導讀者深入理解其各個方面。

## 框架設計理念

Dioxus 的設計理念融合了 React 的聲明式特性和 Rust 的安全性與性能：

- **聲明式 UI** - 使用 RSX (類似 JSX) 語法描述視圖，而不必關心實現細節
- **組件化架構** - 將 UI 分解為可重用、可組合的組件
- **靜態類型安全** - 充分利用 Rust 的類型系統提供編譯時錯誤檢查
- **高性能虛擬 DOM** - 優化的差異比較算法，最小化 DOM 操作
- **跨平台支持** - 單一代碼庫支持 Web、桌面、移動和服務器端渲染

## 核心功能概覽

Dioxus 框架包含以下核心功能，各部分協同工作，形成一個完整的 UI 開發生態系統：

### 1. RSX 語法與 UI 描述

RSX 是 Dioxus 的模板語法，允許開發者使用類似 HTML 的語法在 Rust 代碼中直接描述 UI。

```rust
rsx! {
    div { class: "container",
        h1 { "歡迎使用 Dioxus" }
        p { "一個用 Rust 構建 UI 的現代框架" }
    }
}
```

[閱讀更多關於 RSX 語法的內容](./dioxus-rsx-語法與UI描述.md)

### 2. 組件系統與事件處理

組件是 Dioxus 的基本構建塊，可以接受 Props 並渲染 UI。事件處理系統允許組件響應用戶交互。

```rust
#[derive(Props)]
struct ButtonProps {
    text: String,
    onclick: EventHandler<MouseEvent>,
}

fn Button(props: ButtonProps) -> Element {
    rsx! {
        button {
            onclick: move |evt| props.onclick.call(evt),
            "{props.text}"
        }
    }
}
```

[閱讀更多關於組件系統的內容](./dioxus-組件系統與事件處理.md)

### 3. Hooks 系統與狀態管理

Hooks 允許函數組件使用狀態和其他 Dioxus 特性，提供了一種管理組件狀態和生命週期的統一方式。

```rust
fn Counter() -> Element {
    let mut count = use_signal(|| 0);
    
    rsx! {
        div {
            p { "計數: {count}" }
            button { onclick: move |_| count += 1, "增加" }
        }
    }
}
```

[閱讀更多關於 Hooks 系統的內容](./dioxus-hooks-系統與狀態管理.md)

### 4. 全局狀態管理與上下文

對於跨組件或全局狀態，Dioxus 提供了 Context API 和全局信號系統，使數據可以在整個應用中共享。

```rust
// 全局狀態
static THEME: GlobalSignal<String> = Signal::global(|| "light".to_string());

// Context API
fn App() -> Element {
    let user = use_signal(|| "訪客".to_string());
    provide_context(user);
    
    rsx! { UserProfile {} }
}

fn UserProfile() -> Element {
    let user = use_context::<Signal<String>>();
    rsx! { div { "歡迎, {user}" } }
}
```

[閱讀更多關於全局狀態管理的內容](./dioxus-全局狀態管理與上下文.md)

### 5. 異步功能與資源管理

Dioxus 提供了處理異步操作的專用 Hooks，使數據獲取和異步邏輯變得簡單。

```rust
fn UserList() -> Element {
    let users = use_resource(|| async move {
        reqwest::get("https://api.example.com/users")
            .await?
            .json::<Vec<User>>()
            .await
    });
    
    rsx! {
        match &*users.read_unchecked() {
            Some(Ok(data)) => rsx! { /* 顯示用戶 */ },
            Some(Err(_)) => rsx! { "加載失敗" },
            None => rsx! { "加載中..." },
        }
    }
}
```

[閱讀更多關於異步功能的內容](./dioxus-異步功能與資源管理.md)

## 平台特定功能

Dioxus 支持多個平台，每個平台都有一些特定的 API 和功能：

### Web 平台

```rust
fn main() {
    dioxus_web::launch(App);
}
```

### 桌面平台

```rust
fn main() {
    dioxus_desktop::launch(App);
}
```

### 移動平台

```rust
fn main() {
    dioxus_mobile::launch(App);
}
```

### 服務器端渲染

```rust
fn main() {
    let mut vdom = VirtualDom::new(App);
    let _ = vdom.rebuild();
    println!("{}", dioxus_ssr::render(&vdom));
}
```

## 高級功能

除了核心功能外，Dioxus 還提供了許多高級功能：

### 路由系統

Dioxus Router 提供了聲明式路由功能，支持嵌套路由、參數提取、路由守衛等。

```rust
fn app() -> Element {
    rsx! {
        Router {
            Route { to: "/", Home {} }
            Route { to: "/users", Users {} }
            Route { to: "/users/:id", UserDetail {} }
        }
    }
}
```

### 服務器函數

服務器函數是一種在服務器上執行並可以從客戶端調用的函數，適用於全棧應用。

```rust
#[server]
async fn fetch_data() -> Result<Vec<Item>, ServerFnError> {
    // 在服務器上執行的代碼
    Ok(db::get_items().await?)
}

// 在客戶端調用
let data = use_resource(|| fetch_data());
```

### 和原生 DOM 交互

通過引用系統，可以直接與底層 DOM 或原生 UI 元素交互。

```rust
fn VideoPlayer() -> Element {
    let video_ref = use_ref(|| None);
    
    rsx! {
        video {
            controls: true,
            noderef: video_ref,
            source { src: "/video.mp4" }
        }
        button {
            onclick: move |_| {
                if let Some(video) = video_ref.get() {
                    let _ = video.play();
                }
            },
            "播放"
        }
    }
}
```

## 工具和生態系統

Dioxus 提供了豐富的工具和生態系統：

- **Dioxus CLI** - 項目腳手架、開發服務器、構建工具
- **Dioxus DevTools** - 開發者工具，用於調試和分析
- **Fermi** - 全局狀態管理庫
- **Dioxus Labs 組件庫** - 預製 UI 組件集合

## 學習路徑建議

對於初學者，建議按照以下順序學習 Dioxus：

1. RSX 語法與基礎元素
2. 組件與 Props
3. Hooks 和狀態管理
4. 事件處理
5. 異步數據獲取
6. 全局狀態管理
7. 路由和高級功能

## 文檔導航

以下是 Dioxus 核心功能的詳細文檔：

- [RSX 語法與 UI 描述](./dioxus-rsx-語法與UI描述.md)
- [Hooks 系統與狀態管理](./dioxus-hooks-系統與狀態管理.md)
- [組件系統與事件處理](./dioxus-組件系統與事件處理.md)
- [異步功能與資源管理](./dioxus-異步功能與資源管理.md)
- [全局狀態管理與上下文](./dioxus-全局狀態管理與上下文.md)

## 結語

Dioxus 結合了 React 的易用性和 Rust 的安全性與性能，為開發者提供了一個強大而靈活的 UI 框架。通過本文檔系列，你可以全面了解 Dioxus 的核心功能，並在實際項目中充分利用其潛力。

無論你是構建 Web 應用、桌面應用還是移動應用，Dioxus 都能提供一致的開發體驗和卓越的性能。
