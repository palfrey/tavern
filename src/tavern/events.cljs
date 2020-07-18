(ns tavern.events
  (:require [re-frame.core :as rf]))

(rf/reg-event-db
 ::set-active-panel
 (fn [db [_ active-panel]]
   (assoc db :active-panel active-panel)))
