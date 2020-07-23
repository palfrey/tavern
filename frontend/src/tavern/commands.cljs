(ns tavern.commands
  (:require
   [re-frame.core :as rf]))

(defn send-command [websocket msg]
  (let [data (.stringify js/JSON (clj->js msg))]
    (js/console.log "Sending" data)
    (.send websocket data)))

(defn ping [websocket]
  (if (= (.-readyState websocket) 1)
    (send-command websocket {"kind" "Ping"})
    (js/console.log "Can't ping websocket" websocket)))

(defn list-pubs [websocket]
  (send-command websocket {"kind" "ListPubs"}))

(defn handle-event [data]
  (let [msg (js->clj (.parse js/JSON data) :keywordize-keys true)]
    (js/console.log "decoded" (str msg))
    (case (:kind msg)
      "Pubs" (rf/dispatch [:pubs (:list msg)]))))