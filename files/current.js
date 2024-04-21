/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();

    const list = document.getElementById('sets');
    for (var child of list.children) {
        if (child.classList.contains('active')) {
            let help = document.getElementById('sets-help');
            help.innerText = child.getAttribute("data-summary");
            break;
        }
    }
}

function on_click(item) {
    const list = document.getElementById('sets');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    update_value();

    let help = document.getElementById('sets-help');
    help.innerText = item.getAttribute("data-summary");
}

function update_value() {
    let input = document.getElementById('sets-btn');
    input.value = "";

    const list = document.getElementById('sets');
    for (var child of list.children) {
        if (child.classList.contains("active")) {
            input.value = child.innerText;
            break;
        }
    }
}
