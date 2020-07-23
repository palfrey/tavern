(ns tavern.commands)

(defn list-pubs [websocket]
  (.send websocket (.stringify js/JSON (clj->js {"kind" "ListPubs"}))))

(defn handle-event [data]
  (let [msg (js->clj (.parse js/JSON data) :keywordize-keys true)]
    (js/console.log "decoded" (str msg))))