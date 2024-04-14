/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    let name = document.getElementById('name-field');
    name.addEventListener("input", on_name_change);

    filter_menu();
}

function on_name_change(event) {
    filter_menu();
}

function on_click(item) {
    const list = document.getElementById('names');
    for (var child of list.children) {
        child.classList.remove('active');
        child.setAttribute('aria-current', "false");
    }
    item.classList.add('active');
    item.setAttribute('aria-current', "true");
    update_value();
    // filter_menu();
}

function update_value() {
    const list = document.getElementById('names');
    const len = list.children.length;
    for (let i = 0; i < len; i++) {
        let child = list.children[i];
        if (child.classList.contains('active')) {
            let name = document.getElementById('name-field');
            name.value = child.innerText;
            filter_menu();
            break;
        }
    }
}

// Could use fuzzy matching here. There are a number of javascript libraries for this, e.g.
//    https://github.com/leeoniya/uFuzzy
//    https://www.fusejs.io/    one guy said it’s “garbo”
//    https://github.com/farzher/fuzzysort
// Also could do this on the backend with something like an Ajax request (tho that might
// introduce annoying latency when typing). Not clear how useful this would be tho.
function filter_menu() {
    const MAX_COUNT = 50;
    let name = document.getElementById('name-field').value.toLowerCase();

    let filtered_in = 0;
    const list = document.getElementById('names');
    for (let i = 0; i < list.children.length; i++) {
        let child = list.children[i];
        if (child.innerText.toLowerCase().includes(name)) {
            if (filtered_in < MAX_COUNT) {
                child.removeAttribute('hidden');
            } else {
                child.setAttribute('hidden', "true");
            }
            filtered_in += 1;
        } else {
            child.setAttribute('hidden', "true");
        }
    }

    let ellipsis = document.getElementById('ellipsis');
    if (filtered_in < MAX_COUNT) {
        ellipsis.setAttribute('hidden', "true");
    } else {
        ellipsis.removeAttribute('hidden');
    }
}
