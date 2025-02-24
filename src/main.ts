import { createApp } from "vue";
import App from "./App.vue";
import "./assets/index.css";

// Add dark mode by default to html element
document.documentElement.classList.add('dark')

createApp(App).mount("#app");
