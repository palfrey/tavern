(ns tavern.events
  (:require
   [re-frame.core :as rf]
   [tavern.intervals]
   [tavern.interceptors :as ti]))

(defn getMediaStream []
  (.catch
   (.then (.getUserMedia (-> js/navigator .-mediaDevices) (js-obj "video" true "audio" false))
          (fn [stream]
            (js/console.log "got stream" stream)
            (rf/dispatch [:mediastream stream])))
   (fn [err]
     (js/console.log "usermedia error" err))))

(ti/reg-event-db
 :initialize
 (fn [db [_ _]]
   db))
  ;  (let [id (:peer-id db)
  ;        ]

  ;    (.on peer "call"
  ;         (fn [call]
  ;           (js/console.log "Got a call" call)
  ;           (.on call "error"
  ;                (fn [err]
  ;                  (js/console.log "Call error" err)))
  ;           (.on call "stream"
  ;                (fn [remoteStream]
  ;                  (let [peer-id (.-peer call)]
  ;                    (js/console.log "Got stream for" peer-id remoteStream)
  ;                    (rf/dispatch [:status peer-id :called])
  ;                    (rf/dispatch [:set-stream peer-id remoteStream]))))
  ;           (.answer call (:mediastream db))))
  ;    (assoc db :peer peer))))

(rf/reg-event-fx
 ::set-active-panel
 (fn [{:keys [db]} [_ active-panel]]
   (js/console.log "Navigate to panel" (str active-panel))
   (let [extra
         (case active-panel
           :home-panel
           {:interval-n
            [{:action :start
              :id :now
              :frequency 1000
              :event [:timer]}
             {:action :start
              :id :peers
              :frequency (* 10 1000)
              :event [:peers-timer]}]}
           {})]
     (merge extra
            {:db (assoc db :active-panel active-panel)}))))

(rf/reg-event-db
 :timer
 (fn [db [_ _]]
   (assoc db :time (js/Date.))))

(rf/reg-event-db
 :peers-timer
 (fn [db [_ _]]
   (.listAllPeers
    (:peer db)
    (fn [peerids]
      (println "peers" (js->clj peerids))
      (rf/dispatch [:peers (js->clj peerids)])))
   db))

(ti/reg-event-db
 :peers
 (fn [db [_ peers]]
   (let [existing-peers (get db :peers {})
         existing-peer-ids (set (keys existing-peers))
         peers (set (remove #{(:peer-id db)} peers))
         new-peers (clojure.set/difference peers existing-peer-ids)
         missing-peers (clojure.set/difference existing-peer-ids peers)
         new-peers-as-dict (apply hash-map (flatten (map #(vector % {}) new-peers)))
         revised-peers (merge (apply hash-map (flatten (filter (fn [[k v]] (not (contains? (set missing-peers) k))) existing-peers))) new-peers-as-dict)]
     (if (-> db :mediastream nil? not)
       (doall (for [peer new-peers]
                (do (js/console.log "Calling" peer)
                    (rf/dispatch [:calling peer])
                    (rf/dispatch [:status peer :calling])
                    (let [call (.call (:peer db) peer (:mediastream db))]
                      (.on call "error"
                           (fn [err]
                             (js/console.log "Calling peer error" err)))
                      (.on call "stream"
                           (fn [remoteStream]
                             (js/console.log "Got stream for" peer remoteStream)
                             (rf/dispatch [:set-stream peer remoteStream]))))))))

     (assoc db :peers revised-peers))))

(ti/reg-event-db
 :peer-id
 (fn [db [_ id]]
   (assoc db :peer-id id)))

(ti/reg-event-db
 :mediastream
 (fn [db [_ mediastream]]
   (assoc db :mediastream mediastream)))

(ti/reg-event-db
 :set-stream
 (fn [db [_ peer remoteStream]]
   (js/console.log "Storing stream for" (clj->js peer))
   (assoc-in db [:peers peer :stream] remoteStream)))

(ti/reg-event-db
 :calling
 (fn [db [_ peer]]
   (assoc-in db [:peers peer :calling] true)))

(ti/reg-event-db
 :status
 (fn [db [_ peer kind]]
   (assoc-in db [:peers peer :status] kind)))