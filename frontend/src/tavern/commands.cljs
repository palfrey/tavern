(ns tavern.commands
  (:require
   [re-frame.core :as rf]))

(defn list-pubs [websocket]
  (let [msg {"kind" "ListPubs"}
        data (.stringify js/JSON (clj->js msg))]
    (js/console.log "Sending" data)
    (.send websocket data)))

(defn handle-event [data]
  (let [msg (js->clj (.parse js/JSON data) :keywordize-keys true)]
    (js/console.log "decoded" (str msg))
    (case (:kind msg)
      "Pubs" (rf/dispatch [:pubs (:list msg)]))))