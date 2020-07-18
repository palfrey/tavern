(ns tavern.events
  (:require
   [re-frame.core :as rf]
   [tavern.intervals]
   [tavern.interceptors :as ti]))

(defn getMediaStream []
  (.catch
   (.then (.getUserMedia (-> js/navigator .-mediaDevices) (js-obj "video" true "audio" true))
          (fn [stream]
            (js/console.log "got stream" stream)
            (rf/dispatch [:mediastream stream])))
   (fn [err]
     (js/console.log "usermedia error" err))))

(ti/reg-event-db
 :initialize
 (fn [db [_ _]]
   (let [id (:peer-id db)
         peer (js/Peer. id (js-obj :key "peerjs" "host" "localhost" "port" 9000 "debug" 2
                                   "iceServers" [{"urls" "stun:stun.l.google.com:19302"}]))]
     (if (nil? id)
       (.on peer "open"
            (fn [id]
              (js/console.log "New peer id is" id)
              (rf/dispatch [:peer-id id])))
       (js/console.log "Existing id is" id))

     (.on peer "call"
          (fn [call]
            (js/console.log "Got a call" call)
            (.answer call (:mediastream db))))
     (assoc db :peer peer))))

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
   (let [peers (remove #{(:peer-id db)} peers)]
     (doall (for [peer peers]
              (do (js/console.log "Calling" peer)
                  (.call (:peer db) peer (:mediastream db)))))
     (assoc db :peers peers))))

(ti/reg-event-db
 :peer-id
 (fn [db [_ id]]
   (assoc db :peer-id id)))

(ti/reg-event-db
 :mediastream
 (fn [db [_ mediastream]]
   (assoc db :mediastream mediastream)))