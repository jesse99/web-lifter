/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
    enable_menu();
}

function on_click(item) {
    const list = document.getElementById('exercises');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    enable_menu();
}

function on_enable() {
    const list = document.getElementById('exercises');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            child.classList.remove('text-black-50');
            update_value();
            enable_menu();
            break;
        }
    }
}

function on_disable() {
    const list = document.getElementById('exercises');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            child.classList.add('text-black-50');
            update_value();
            enable_menu();
            break;
        }
    }
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
            enable_menu();
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
            enable_menu();
            break;
        }
    }
}

function update_value() {
    let exercises = "";
    let disabled = "";
    const list = document.getElementById('exercises');
    for (var child of list.children) {
        if (exercises) {
            exercises += "\t";
            disabled += "\t";
        }
        exercises += child.innerText;
        if (child.classList.contains("text-black-50")) {
            disabled += "true";
        } else {
            disabled += "false";
        }
    }
    let input = document.getElementById('exercises-btn');
    input.value = exercises;

    input = document.getElementById('disabled-btn');
    input.value = disabled;
}

function enable_menu() {
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

            let dbutton = document.getElementById('disable-btn');
            let ebutton = document.getElementById('enable-btn');
            if (child.classList.contains("text-black-50")) {
                dbutton.classList.add('disabled');
                ebutton.classList.remove('disabled');
            } else {
                dbutton.classList.remove('disabled');
                ebutton.classList.add('disabled');
            }
            break;
        }
    }
}
