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
    (js/console.log "Can't ping websocket" websocket))
  (= (.-readyState websocket) 1))

(defn list-pubs [websocket]
  (send-command websocket {"kind" "ListPubs"}))

(defn create-pub [websocket name]
  (send-command websocket {"kind" "CreatePub" "name" name}))

(defn delete-pub [websocket pub_id]
  (send-command websocket {"kind" "DeletePub" "pub_id" pub_id}))

(defn join-pub [websocket pub_id]
  (send-command websocket {"kind" "JoinPub" "pub_id" pub_id}))

(defn leave-pub [websocket]
  (send-command websocket {"kind" "LeavePub"}))

(defn list-tables [websocket pub_id]
  (send-command websocket {"kind" "ListTables" "pub_id" pub_id}))

(defn create-table [websocket pub_id name]
  (send-command websocket {"kind" "CreateTable" "pub_id" pub_id "name" name}))

(defn join-table [websocket table_id]
  (send-command websocket {"kind" "JoinTable" "table_id" table_id}))

(defn get-person [websocket user_id]
  (send-command websocket {"kind" "GetPerson" "user_id" user_id}))

(defn handle-event [data]
  (let [msg (js->clj (.parse js/JSON data) :keywordize-keys true)]
    (js/console.log "decoded" (str msg))
    (case (:kind msg)
      "Pubs"
      (rf/dispatch [:pubs (apply hash-map (flatten (map #(vector (:id %) %) (:list msg))))])
      "Tables"
      (rf/dispatch [:tables (apply hash-map (flatten (map #(vector (:id %) %) (:list msg))))])
      "Pong" (do)
      "Pub" (rf/dispatch [:pub (:data msg)])
      "Person" (rf/dispatch [:person (:data msg)]))))