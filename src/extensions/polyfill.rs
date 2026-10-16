/// Generates the WebExtensions API polyfill injected into web pages and extension contexts
/// This provides compatibility for both Google Chrome (`chrome.*`) and Mozilla Firefox (`browser.*`).
pub fn generate_webextensions_polyfill(extension_id: &str, extension_name: &str) -> String {
    format!(
        r#"(function() {{
    if (window.__browsy_extension_injected) return;
    window.__browsy_extension_injected = true;

    const EXT_ID = "{extension_id}";
    const EXT_NAME = "{extension_name}";

    const listeners = [];
    const storageStore = {{}};

    const webExtAPI = {{
        runtime: {{
            id: EXT_ID,
            getURL: function(path) {{
                if (path.startsWith("/")) path = path.slice(1);
                return "browsy-extension://" + EXT_ID + "/" + path;
            }},
            sendMessage: function(msg, callback) {{
                console.log("[WebExtension:" + EXT_NAME + "] sendMessage:", msg);
                // Dispatch to listeners asynchronously
                setTimeout(() => {{
                    for (const listener of listeners) {{
                        try {{
                            listener(msg, {{ id: EXT_ID }}, (response) => {{
                                if (callback) callback(response);
                            }});
                        }} catch (e) {{
                            console.error("[WebExtension] Listener error:", e);
                        }}
                    }}
                }}, 0);
            }},
            onMessage: {{
                addListener: function(listener) {{
                    listeners.push(listener);
                }},
                removeListener: function(listener) {{
                    const idx = listeners.indexOf(listener);
                    if (idx !== -1) listeners.splice(idx, 1);
                }}
            }}
        }},
        storage: {{
            local: {{
                get: function(keys, callback) {{
                    const result = {{}};
                    if (typeof keys === 'string') {{
                        result[keys] = storageStore[keys];
                    }} else if (Array.isArray(keys)) {{
                        for (const k of keys) result[k] = storageStore[k];
                    }} else if (keys === null || keys === undefined) {{
                        Object.assign(result, storageStore);
                    }}
                    if (callback) callback(result);
                    return Promise.resolve(result);
                }},
                set: function(items, callback) {{
                    Object.assign(storageStore, items);
                    if (callback) callback();
                    return Promise.resolve();
                }},
                remove: function(keys, callback) {{
                    const list = Array.isArray(keys) ? keys : [keys];
                    for (const k of list) delete storageStore[k];
                    if (callback) callback();
                    return Promise.resolve();
                }}
            }},
            sync: {{
                get: function(keys, callback) {{ return webExtAPI.storage.local.get(keys, callback); }},
                set: function(items, callback) {{ return webExtAPI.storage.local.set(items, callback); }},
                remove: function(keys, callback) {{ return webExtAPI.storage.local.remove(keys, callback); }}
            }}
        }},
        tabs: {{
            query: function(queryInfo, callback) {{
                const mockTab = [{{ id: 1, active: true, url: window.location.href, title: document.title }}];
                if (callback) callback(mockTab);
                return Promise.resolve(mockTab);
            }},
            sendMessage: function(tabId, message, callback) {{
                return webExtAPI.runtime.sendMessage(message, callback);
            }},
            create: function(createProperties, callback) {{
                if (createProperties.url) {{
                    window.open(createProperties.url, '_blank');
                }}
                if (callback) callback({{ id: Date.now(), url: createProperties.url }});
                return Promise.resolve({{ id: Date.now(), url: createProperties.url }});
            }}
        }},
        action: {{
            setTitle: function(details) {{ console.log("[WebExtension] setTitle:", details); }},
            setBadgeText: function(details) {{ console.log("[WebExtension] setBadgeText:", details); }}
        }},
        browserAction: {{
            setTitle: function(details) {{ console.log("[WebExtension] setTitle:", details); }},
            setBadgeText: function(details) {{ console.log("[WebExtension] setBadgeText:", details); }}
        }}
    }};

    // Support Chrome extension namespace
    window.chrome = window.chrome || webExtAPI;
    // Support Firefox / WebExtension browser standard namespace
    window.browser = window.browser || webExtAPI;

    console.log("[Browsy] WebExtensions Polyfill loaded for: " + EXT_NAME);
}})();"#
    )
}
