/* eslint-env es6 */
/* eslint no-undef: "off" */
/* eslint no-console: "warn" */
"use strict";

let start_time = undefined;
let deadline = undefined;
let timer_id = undefined;   // can be used with clearInterval

function format_int(x) {
    return x.toLocaleString(undefined, { maximumFractionDigits: 0 });
}

// since midnight, 1 Jan 1970
function seconds() {
    return new Date().getTime() / 1000;
}

function on_next(event) {
    const body = document.getElementById('body');
    const rest = parseInt(body.getAttribute("data-rest"));

    if (deadline === undefined) {
        if (rest > 0) {
            // Start resting
            let button = document.getElementById('next_button');
            button.innerHTML = "Stop Resting";

            start_time = seconds();
            deadline = start_time + rest;
            update_label();
            timer_id = setInterval(on_timer, 1000); // ms
        } else {
            // No need to rest so advance to next set
            post_next_set();
        }
    } else {
        // User said he is done resting
        clearInterval(timer_id);
        timer_id = undefined;
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
function update_label() {
    const current = seconds();
    let label = document.getElementById('timer_text');
    const remaining = deadline - current;
    if (current < deadline) {
        // console.log(`remaining: ${remaining}`);
        label.innerHTML = format_int(remaining) + " secs";  // TODO use a friendly_time_units function
        label.style.color = "red";
    } else if (current < deadline + 2) {
        label.innerHTML = "Done";
        label.style.color = "green";
    } else {
        label.innerHTML = "+" + format_int(-remaining) + " secs";
        label.style.color = "green";
    }
}

function on_timer() {
    update_label();
}
