/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
    enable_menu();
}

function on_click(item) {
    const list = document.getElementById('list');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    enable_menu();

    let text = item.getAttribute("data-help");
    if (text) {
        let help = document.getElementById('list-help');
        help.innerText = text;
    }
}

function has_block(name) {
    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.innerText === name) {
            return true;
        }
    }
    return false;
}

function resort() {
    const list = document.getElementById('list');
    [...list.children]
        .sort((x, y) => {
            return x.innerText > y.innerText ? 1 : -1
        })
        .forEach(node => list.appendChild(node));
}

function on_add(prefix) {
    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len + 1; i++) {
        const name = `${prefix} ${i + 1}`;
        if (!has_block(name)) {
            let item = document.createElement("li");
            item.classList.add("list-group-item");
            item.setAttribute("onclick", "on_click(this)");
            item.innerText = name;
            list.appendChild(item);

            resort()
            update_value();
            break;
        }
    }
}

function on_delete() {
    const list = document.getElementById('list');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            list.removeChild(child);
            update_value();
            enable_menu();
            break;
        }
    }
}

function update_value() {
    let names = "";
    const list = document.getElementById('list');
    for (var child of list.children) {
        if (names) {
            names += "¦";
        }
        names += child.innerText;
    }
    let input = document.getElementById('list-button');
    input.value = names;
}

function enable_menu() {
    const list = document.getElementById('list');
    const len = list.children.length;

    for (var name of ["delete-btn"]) {
        let button = document.getElementById(name);
        button.classList.add('disabled');
    }

    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            let button = document.getElementById('delete-btn');
            button.classList.remove('disabled');
            break;
        }
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
