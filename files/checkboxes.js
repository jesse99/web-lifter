/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

function on_loaded() {
    update_value();
}

function on_click() {
    update_value();
}

function update_value() {
    let input = document.getElementById('check-values');
    input.value = "";

    const checkboxes = document.querySelectorAll('input[type="checkbox"]');
    for (var checkbox of checkboxes) {
        if (checkbox.checked) {
            if (input.value) {
                input.value += "¦";
            }
            input.value += checkbox.value;
        }
    }
}

window.addEventListener('DOMContentLoaded', on_loaded);
