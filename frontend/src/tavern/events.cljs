(ns tavern.events
  (:require
   [re-frame.core :as rf]
   [tavern.commands :as commands]
   [tavern.intervals]
   [tavern.interceptors :as ti]))

(defn getMediaStream []
  (.catch
   (.then (.getUserMedia (-> js/navigator .-mediaDevices) (js-obj "video" true "audio" false "width" 320))
          (fn [stream]
            (js/console.log "got stream" stream)
            (rf/dispatch [:mediastream stream])))
   (fn [err]
     (js/console.log "usermedia error" err))))

(ti/reg-event-db
 :initialize
 (fn [db [_ _]]
   (let [db (if (contains? db :peer-id) db (assoc db :peer-id (random-uuid)))
         websocket (js/WebSocket. (str "wss://localhost:8000/ws/" (str (:peer-id db))))
         db (assoc db :websocket websocket)]
     (.addEventListener
      websocket "open"
      (fn [event]
        (js/console.log "Socket open")
        (commands/list-pubs websocket)))
     (.addEventListener
      websocket "message"
      (fn [event]
        (commands/handle-event (.-data event))))
     (.addEventListener
      websocket "error"
      (fn [event]
        (js/console.log "Error" event)))
     db)))

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
  ;  (.listAllPeers
  ;   (:peer db)
  ;   (fn [peerids]
  ;     (println "peers" (js->clj peerids))
  ;     (rf/dispatch [:peers (js->clj peerids)])))
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

(ti/reg-event-db
 :pubs
 (fn [db [_ pubs]]
   (assoc db :pubs pubs)))