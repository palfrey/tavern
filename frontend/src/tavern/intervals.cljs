(ns tavern.intervals
  (:require
   [re-frame.core :as rf]))

(defonce interval-handler                ;; notice the use of defonce
  (let [live-intervals (atom {})]        ;; storage for live intervals
    (fn handler [{:keys [action id frequency event] :as effect}]     ;; the effect handler
      (js/console.log "effect" (str effect))
      (condp = action
        :clean   (doall                ;; <--- new. clean up all existing
                  (map #(handler {:action :end :id %1}) (keys @live-intervals)))
        :start   (do
                   (swap! live-intervals assoc id (js/setInterval #(rf/dispatch event) frequency))
                   (rf/dispatch event))
        :end     (do (js/clearInterval (get @live-intervals id))
                     (swap! live-intervals dissoc id))))))

(defn interval-n-handler [intervals]
  (doall (map interval-handler intervals)))

  ;; when this code is reloaded `:clean` existing intervals
(interval-handler {:action :clean})

(rf/reg-fx
 :interval
 interval-handler)

(rf/reg-fx
 :interval-n
 interval-n-handler)
