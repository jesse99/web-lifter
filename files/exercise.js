/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

let start_time = 0;
let deadline = 0;
let timer_id = 0;   // can be used with clearInterval

function format_int(x) {
    x.toLocaleString(undefined, { maximumFractionDigits: 0 })
}

// since midnight, 1 Jan 1970
function seconds() {
    return new Date().getTime() / 1000;
}

function on_next(event) {
    const body = document.getElementById('body');
    const rest = parseInt(body.getAttribute("data-rest"));

    if (rest > 0) {
        start_time = seconds();
        deadline = start_time + rest;
        timer_id = setInterval(on_timer, 1000); // ms
    } else {
        post_next_set();
    }
}

// We can send a POST with XMLHttpRequest but that won't load a new page so what we do
// is dynamically create a form and submit that. This is based on 
// https://stackoverflow.com/questions/17378619/navigate-to-another-page-with-post-request-through-link.
function post_next_set() {
    const body = document.getElementById('body');
    const workout = body.getAttribute("data-workout");
    const exercise = body.getAttribute("data-exercise");

    // var payload = {
    //     name: 'John',
    //     time: '2pm'
    // };
    var form = document.createElement('form');
    form.style.visibility = 'hidden'; // no user interaction is necessary
    form.method = 'POST'; // forms by default use GET query strings
    form.action = `/exercise/${workout}/${exercise}/next-set`;
    // for (key in Object.keys(payload)) {
    //     var input = document.createElement('input');
    //     input.name = key;
    //     input.value = payload[key];
    //     form.appendChild(input); // add key/value pair to form
    // }
    document.body.appendChild(form); // forms cannot be submitted outside of body
    form.submit(); // send the payload and navigate
}

// Note that we've told the browser to call us every second but that won't be perfectly
// reliable and errors will accumulate so we get the current time instead of relying on
// that.
function on_timer() {
    const current = seconds();
    if (current < deadline) {
        const remaining = deadline - current;
        // console.log(`${remaining} seconds remaining`);

        let label = document.getElementById('timer_text');
        label.innerHTML = format_int(remaining) + " secs";
    } else {
        post_next_set();
    }
}
