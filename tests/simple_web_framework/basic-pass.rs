pub mod basic_web_framework;

fn main() {
    #[allow(unused_imports)]
    use basic_web_framework::prelude::*;

    let chosen_number = 42_u32;
    let some_string: String = String::from("boo!");
    let some_html_value = html! { <span>{"Some html value"}</span> };
    let world = "world";
    let name_expr = "name_expr";
    let var_class2 = "var_class2";
    let style = "style";

    let _ = html! {
        <>
        <div class="hello" style={world}>
            <span class=["class1", var_class2] {style}>
                {"The magic number is "}
                // same as: {format_args!("{:x}", chosen_number)}
                ("{:x}", chosen_number)
                <br {name_expr}="42" numeric=42 {"block_name"}={"block_value"} />
                {some_html_value}
                {some_string}
                <MyComponent />
                <MyComponent<u32> />
            </span>
        </div>
        <span />
        </>
    };
}
