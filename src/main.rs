mod components;
mod invoke;
mod pages;
mod route;

fn main() {
    yew::Renderer::<route::Main>::new().render();
}
