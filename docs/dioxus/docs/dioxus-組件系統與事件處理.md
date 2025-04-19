# Dioxus 組件系統與事件處理

組件是 Dioxus 應用程序的基本構建塊，它們封裝 UI 和行為，使代碼更易於組織和維護。本文檔詳細介紹 Dioxus 的組件系統和事件處理機制。

## 1. 組件基礎

### 1.1 組件定義

在 Dioxus 中，組件是返回 `Element` 類型的 Rust 函數，通常使用 UpperCamelCase 命名：

```rust
// 基本組件定義
fn Button() -> Element {
    rsx! {
        button {
            class: "primary-button",
            "點擊我"
        }
    }
}
```

組件可以接受輸入參數（稱為 Props）或無參數：

```rust
// 無參數組件
fn Logo() -> Element {
    rsx! {
        img { src: "/logo.png", alt: "公司標誌" }
    }
}

// 帶參數組件
fn Greeting(name: String) -> Element {
    rsx! {
        h1 { "你好，{name}!" }
    }
}
```

### 1.2 組件使用

在 RSX 中可以像 HTML 元素一樣使用組件：

```rust
fn App() -> Element {
    rsx! {
        div {
            Logo {}
            Greeting { name: "小明".to_string() }
        }
    }
}
```

## 2. Props 系統

Props（Properties）是傳遞給組件的參數，Dioxus 通過 `#[derive(Props)]` 宏簡化了 Props 的定義。

### 2.1 定義 Props

```rust
// 使用 derive 宏定義 Props
#[derive(Props, PartialEq, Clone)]
pub struct ButtonProps {
    // 必填屬性
    label: String,
    
    // 可選屬性，使用 Option
    #[props(default)]
    color: Option<String>,
    
    // 帶默認值的屬性
    #[props(default = "default_width()")]
    width: u32,
    
    // 事件處理器
    onclick: EventHandler<MouseEvent>,
    
    // 子元素
    children: Element,
}

fn default_width() -> u32 {
    100
}
```

### 2.2 使用 Props 的組件

```rust
fn Button(props: ButtonProps) -> Element {
    let color = props.color.unwrap_or("blue".to_string());
    
    rsx! {
        button {
            class: "btn btn-{color}",
            style: "width: {props.width}px",
            onclick: move |evt| props.onclick.call(evt),
            
            // 渲染標籤文字
            span { "{props.label}" }
            
            // 渲染子元素
            {props.children}
        }
    }
}
```

### 2.3 使用帶 Props 的組件

```rust
fn App() -> Element {
    rsx! {
        Button {
            label: "提交".to_string(),
            color: Some("green".to_string()),
            width: 200,
            onclick: move |_| println!("按鈕被點擊了"),
            
            // 子元素作為內容傳遞
            span { class: "icon", "📤" }
        }
    }
}
```

## 3. 事件處理

Dioxus 提供了豐富的事件處理機制，使組件可以響應用戶交互。

### 3.1 基本事件監聽

事件處理器是以 `on` 開頭的特殊屬性，它們接受閉包作為值：

```rust
rsx! {
    button {
        onclick: move |event| println!("按鈕被點擊: {:?}", event),
        "點擊我"
    }
    
    input {
        oninput: move |evt| {
            if let Some(value) = evt.value() {
                println!("輸入值: {}", value);
            }
        }
    }
}
```

### 3.2 常見事件類型

Dioxus 支持多種 DOM 事件：

- **滑鼠事件**：`onclick`, `onmousedown`, `onmouseup`, `onmouseover` 等
- **鍵盤事件**：`onkeydown`, `onkeyup`, `onkeypress` 等
- **表單事件**：`oninput`, `onchange`, `onsubmit` 等
- **焦點事件**：`onfocus`, `onblur` 等
- **剪貼板事件**：`oncopy`, `oncut`, `onpaste` 等

每種事件類型都有相應的數據結構：

```rust
// 滑鼠事件範例
onclick: move |evt: MouseEvent| {
    println!("點擊座標: {:?}", evt.data().coordinates.client);
    println!("按鍵: {:?}", evt.data().trigger_button);
}

// 鍵盤事件範例
onkeydown: move |evt: KeyboardEvent| {
    println!("按鍵: {}", evt.key());
    println!("是否按下Ctrl: {}", evt.modifiers().ctrl);
}
```

### 3.3 事件傳播

事件會從觸發元素向上傳播（冒泡）。可以通過 `stop_propagation()` 方法阻止傳播：

```rust
rsx! {
    div {
        onclick: move |_| println!("外層 div 被點擊"),
        
        button {
            onclick: move |evt| {
                evt.stop_propagation();
                println!("按鈕被點擊");
            },
            "點擊按鈕"
        }
    }
}
```

### 3.4 阻止默認行為

通過 `prevent_default()` 方法可以阻止事件的默認行為：

```rust
rsx! {
    a {
        href: "https://example.com",
        onclick: move |evt| {
            evt.prevent_default();
            println!("連結點擊，但頁面不會跳轉");
        },
        "點擊我"
    }
}
```

## 4. 事件處理器作為 Props

組件可以接受事件處理器作為 Props，實現回調功能。

### 4.1 定義事件處理器 Props

```rust
#[derive(Props, PartialEq, Clone)]
pub struct FancyButtonProps {
    // 標準事件處理器類型
    onclick: EventHandler<MouseEvent>,
    
    // 自定義數據事件處理器
    onselect: EventHandler<String>,
    
    // 返回值回調
    on_count: Callback<u32, u32>,
}
```

### 4.2 在組件中使用事件處理器

```rust
fn FancyButton(props: FancyButtonProps) -> Element {
    rsx! {
        button {
            // 轉發標準事件
            onclick: move |evt| props.onclick.call(evt),
            
            // 調用帶自定義數據的事件處理器
            ondblclick: move |_| props.onselect.call("選項A".to_string()),
            
            // 使用返回值回調
            oncontextmenu: move |evt| {
                evt.prevent_default();
                let result = props.on_count.call(5);
                println!("計算結果: {}", result);
            },
            
            "多功能按鈕"
        }
    }
}
```

### 4.3 使用帶事件處理器的組件

```rust
fn App() -> Element {
    let mut count = use_signal(|| 0);
    
    rsx! {
        FancyButton {
            // 標準事件處理
            onclick: move |_| println!("按鈕點擊"),
            
            // 自定義數據處理
            onselect: move |value| println!("選擇了: {}", value),
            
            // 返回值處理
            on_count: move |input| {
                let new_count = count() + input;
                count.set(new_count);
                new_count
            }
        }
        
        div { "當前計數: {count}" }
    }
}
```

## 5. 組件組合

Dioxus 鼓勵將複雜界面分解為可重用的小型組件。

### 5.1 組件嵌套

```rust
fn Header() -> Element {
    rsx! {
        header {
            Logo {}
            Navigation {}
        }
    }
}

fn Content() -> Element {
    rsx! {
        main {
            h1 { "主要內容" }
            p { "歡迎來到我們的網站" }
        }
    }
}

fn Footer() -> Element {
    rsx! {
        footer {
            "版權所有 © 2023"
        }
    }
}

fn App() -> Element {
    rsx! {
        div { class: "app-container",
            Header {}
            Content {}
            Footer {}
        }
    }
}
```

### 5.2 組件子元素

可以通過 Props 中的 `children` 字段傳遞子元素：

```rust
#[derive(Props, PartialEq, Clone)]
pub struct CardProps {
    title: String,
    children: Element,
}

fn Card(props: CardProps) -> Element {
    rsx! {
        div { class: "card",
            div { class: "card-header",
                h3 { "{props.title}" }
            }
            div { class: "card-body",
                // 渲染子元素
                {props.children}
            }
        }
    }
}

fn App() -> Element {
    rsx! {
        Card { title: "用戶資料".to_string(),
            div {
                p { "姓名: 張三" }
                p { "年齡: 28" }
            }
        }
    }
}
```

## 6. 組件最佳實踐

### 6.1 保持組件專注

每個組件應該只負責一個功能，遵循單一責任原則：

- **過小的組件**：可能導致過度拆分和管理複雜
- **過大的組件**：難以維護和理解
- **恰當大小**：能夠獨立測試和理解，但不會過度碎片化

### 6.2 合理使用 Props

- 使用明確的類型而非 `Option` 表示必要參數
- 為常用配置提供合理默認值
- 考慮 Props 的可組合性和靈活性

### 6.3 狀態提升

當多個組件需要共享狀態時，將狀態提升到它們的共同父組件：

```rust
fn App() -> Element {
    // 共享狀態在父組件中管理
    let mut selected_item = use_signal(|| None::<String>);
    
    rsx! {
        // 將狀態與更新函數傳給子組件
        ItemList {
            items: vec!["項目1", "項目2", "項目3"],
            selected: selected_item(),
            on_select: move |item| selected_item.set(Some(item)),
        }
        
        DetailView {
            item: selected_item(),
        }
    }
}
```

### 6.4 組件文檔

為組件添加良好的文檔，說明其用途、Props 和使用方式：

```rust
/// Card 組件顯示帶標題和內容的卡片界面
///
/// # Props
/// * `title` - 卡片標題
/// * `width` - 卡片寬度 (可選, 默認 300px)
/// * `children` - 卡片內容
///
/// # 範例
/// ```rust
/// rsx! {
///     Card { title: "資訊".to_string(),
///         p { "一些重要信息" }
///     }
/// }
/// ```
#[derive(Props, PartialEq, Clone)]
pub struct CardProps {
    title: String,
    #[props(default = 300)]
    width: u32,
    children: Element,
}
```

## 7. 組件性能優化

### 7.1 精細控制重新渲染

使用 Props 的 `PartialEq` 實現控制組件何時重新渲染：

```rust
// 自定義比較邏輯，只有在重要屬性變化時才重新渲染
impl PartialEq for ComplexProps {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
        // 忽略不影響渲染的屬性
    }
}
```

### 7.2 使用 memo 緩存複雜計算

```rust
fn ExpensiveComponent() -> Element {
    let items = use_signal(|| get_large_item_list());
    
    // 緩存複雜計算，只在 items 變化時重新計算
    let processed_items = use_memo(move |_| {
        items.read().iter()
            .filter(|item| item.is_valid)
            .map(|item| process_item(item))
            .collect::<Vec<_>>()
    });
    
    rsx! {
        for item in processed_items.read().iter() {
            DisplayItem { item: item.clone() }
        }
    }
}
```

Dioxus 的組件系統和事件處理機制提供了構建交互式應用程序的強大基礎。通過合理組織組件結構、適當處理事件，並遵循最佳實踐，可以創建出高效、可維護且用戶友好的應用程序。
