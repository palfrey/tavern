(ns tavern.interceptors
  (:require
   [akiroz.re-frame.storage :refer [persist-db-keys]]
   [re-frame.core :refer [reg-event-fx]]))

(def key-interceptor
  (persist-db-keys :tavern [:name :pub :table :peer-id]))

(defn reg-event-db
  [event-id handler]
  (reg-event-fx
   event-id
   [key-interceptor]
   (fn [{:keys [db]} event-vec]
     (let [new-db (handler db event-vec)]
       (if (nil? new-db)
         (do (println "Got nil from " event-id "handler" (str handler))
             {})
         {:db new-db})))))