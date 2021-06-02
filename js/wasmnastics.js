params_register_js_plugin = function (importObject) {
    // === Object tools ===
    importObject.env.null = function () {
        return js_object(null);
    }
    importObject.env.type_of = function (obj) {
        return js_object(typeof get_js_object(obj));
    }
    importObject.env.set_field_any = function (obj, key, val) {
        let real_obj = get_js_object(obj);
        let real_key = get_js_object(key);
        let real_val = get_js_object(val);
        real_obj[real_key] = real_val;
    }
    importObject.env.as_string = function (obj) {
        return js_object(get_js_object(obj).toString());
    }
    // we need a seperate function for each primitive type
    let primitives = [
        ["u8", "number"],
        ["u16", "number"],
        ["u32", "number"],
        ["u64", "number"],
        ["usize", "number"],
        ["i8", "number"],
        ["i16", "number"],
        ["i32", "number"],
        ["i64", "number"],
        ["isize", "number"],
        ["f32", "number"],
        ["f64", "number"],
        ["bool", "boolean"],
    ];

    function primitive(p) {
        return js_object(p);
    }
    for (let [prim, typeof_type] of primitives) {
        importObject.env["primitive_to_js_" + prim] = primitive;
        // make closures
        function from_primitive(p) {
            let inner = get_js_object(p);

            let out;
            if (typeof inner != typeof_type) {
                out = null;
            } else {
                out = {
                    some: inner
                };
            }
            return js_object(out);
        }
        importObject.env["primitive_from_js_" + prim] = from_primitive;
    }


    importObject.env.array = function () {
        return js_object([]);
    }

    importObject.env.try_get_field = function (obj, key) {
        try {
            obj = get_js_object(obj);
            key = get_js_object(key);
            let val = obj[key];

            /// Return a LongOption
            if (val === undefined) {
                return js_object(null);
            } else {
                return js_object({
                    some: val
                });
            }
        } catch (e) {
            return js_object(null);
        }
    }
    importObject.env.equals = function (a, b, triple) {
        a = get_js_object(a);
        b = get_js_object(b);
        if (triple) {
            return a === b;
        } else {
            return a == b;
        }
    }
    importObject.env.has_field = function (obj, buf, length) {
        let field_name = UTF8ToString(buf, length);
        return js_objects[obj][field_name] !== undefined;
    }



    // === Storage ===
    importObject.env.storage_save = function (key, val) {
        try {
            key = get_js_object(key);
            val = get_js_object(val);
            localStorage.setItem(key, val);
            return js_object({
                ok: null
            });
        } catch (e) {
            return js_object({
                err: "Couldn't save to localstorage: " + e.toString()
            });
        }
    }
    importObject.env.storage_load = function (key) {
        try {
            key = get_js_object(key);
            let found = localStorage.getItem(key);

            let out = (found === null) ? {
                err: `Couldn't find key \`${key}\` in localstorage`
            } : {
                ok: found
            };
            return js_object(out);
        } catch (e) {
            return js_object({
                err: "Couldn't load from localstorage: " + e.toString()
            });
        }
    }

    // === Clipboard ===
    importObject.env.clipboard_get = function () {
        let waiter = waitify(navigator.clipboard.readText());
        return js_object(waiter);
    }

    importObject.env.clipboard_set = function (text) {
        let waiter = waitify(navigator.clipboard.writeText(get_js_object(text)));
        return js_object(waiter);
    }

    importObject.env.console_log = function (obj) {
        console.log(get_js_object(obj));
    };
};

miniquad_add_plugin({
    register_plugin: params_register_js_plugin,
    name: "wasmnastics",
    version: "0.1.0",
})

/**
 * Turn a Promise into an object that will be updated later.
 * 
 * The returned object starts with `waiting = true`. Once the promise
 * resolves, `waiting = false` and `value = <the value>`.
 * 
 * In case of error, it will be console logged and `waiting` will never be set to `false`.
 * 
 * @param {Promise} promise Once this resolves, its value will be filled in.
 */
function waitify(promise) {
    let out = {
        waiting: true
    };

    promise.then(
        (val) => {
            out.waiting = false;
            out.value = val;
        },
        (oh_no) => {
            console.log(oh_no);
            // pretend like it's still waiting.
        }
    );

    return out;
}