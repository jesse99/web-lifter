/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
    enable();
}

function on_click(item) {
    const list = document.getElementById('exercises');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    enable();
}

function on_move_down() {
    const list = document.getElementById('exercises');
    const len = list.children.length;
    for (let i = 0; i < len - 1; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            if (i + 2 < len) {
                let target = list.children[i + 2];
                list.insertBefore(child, target);
            } else {
                list.insertBefore(child, null);
            }
            update_value();
            enable();
            break;
        }
    }
}

function on_move_up() {
    const list = document.getElementById('exercises');
    const len = list.children.length;
    for (let i = 1; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            if (i - 1 >= 0) {
                let target = list.children[i - 1];
                list.insertBefore(child, target);
            }
            update_value();
            enable();
            break;
        }
    }
}

function update_value() {
    let value = "";
    const list = document.getElementById('exercises');
    for (var child of list.children) {
        if (value) {
            value += "\t";
        }
        value += child.innerText;
    }
    let input = document.getElementById('exercises-btn');
    input.value = value;
}

function enable() {
    const list = document.getElementById('exercises');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            let button = document.getElementById('up-btn');
            if (i == 0) {
                button.classList.add('disabled');
            } else {
                button.classList.remove('disabled');
            }

            button = document.getElementById('down-btn');
            if (i == len - 1) {
                button.classList.add('disabled');
            } else {
                button.classList.remove('disabled');
            }
            break;
        }
    }
}
