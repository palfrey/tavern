(ns tavern.events
  (:require [re-frame.core :as rf]))

(rf/reg-event-db
 :initialize
 (fn [_ _]
   {:time (js/Date.)
    :peer (js/Peer. nil (js-obj :key "peerjs" "host" "localhost" "port" 9000))}))

(rf/reg-event-db
 ::set-active-panel
 (fn [db [_ active-panel]]
   (assoc db :active-panel active-panel)))

(rf/reg-event-db
 :timer
 (fn [db [_ new-time]]
   (assoc db :time new-time)))