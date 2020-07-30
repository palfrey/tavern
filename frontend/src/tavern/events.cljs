(ns tavern.events
  (:require
   [re-frame.core :as rf]
   [tavern.commands :as commands]
   [tavern.intervals]
   [tavern.interceptors :as ti]
   [tavern.video :as video]))

(defn create-ws [peer-id]
  (let [websocket (js/WebSocket. (str "wss://localhost:8000/ws/" peer-id))]
    (.addEventListener
     websocket "open"
     (fn [event]
       (js/console.log "Socket open")
       (commands/get-person websocket peer-id)))
    (.addEventListener
     websocket "message"
     (fn [event]
       (commands/handle-event (.-data event))))
    (.addEventListener
     websocket "error"
     (fn [event]
       (js/console.log "Error" event)))
    (.addEventListener
     websocket "close"
     (fn [event]
       (js/console.log "close" event)))
    websocket))

(ti/reg-event-db
 :create-ws
 (fn [db _]
   (let [db (if (contains? db :peer-id) db (assoc db :peer-id (str (random-uuid))))
         existing-socket (:websocket db)
         db (if (or (nil? existing-socket) (contains? #{2 3} (.-readyState existing-socket))) (assoc db :websocket (create-ws (-> db :peer-id str))) db)]
     db)))

(rf/reg-event-fx
 :set-active-panel
 (fn [{:keys [db]} [_ active-panel]]
   (js/console.log "Navigate to panel" (str active-panel))
   (let [extra
         (case active-panel
           :home-panel
           (do
             (commands/list-pubs (:websocket db))
             {:interval-n
              [{:action :start
                :id :now
                :frequency 1000
                :event [:timer]}
               {:action :start
                :id :peers
                :frequency (* 10 1000)
                :event [:ping]}]})
           {})]
     (merge extra
            {:db (assoc db :active-panel active-panel)}))))

(rf/reg-event-db
 :timer
 (fn [db [_ _]]
   (assoc db :time (js/Date.))))

(ti/reg-event-db
 :peer-id
 (fn [db [_ id]]
   (assoc db :peer-id id)))

(ti/reg-event-db
 :mediastream
 (fn [db [_ mediastream]]
   (assoc db :mediastream mediastream)))

(ti/reg-event-db
 :peer
 (fn [db [_ peer connection]]
   (js/console.log "Storing stream for" (clj->js peer))
   (assoc-in db [:peers peer :connection] connection)))

(ti/reg-event-db
 :msg
 (fn [db [_ peer msg]]
   (if-let [conn (get-in db [:peers peer :connection])]
     (video/handle-msg conn (.parse js/JSON msg)))
   db))

(ti/reg-event-db
 :pubs
 (fn [db [_ pubs]]
   (assoc db :pubs pubs)))

(ti/reg-event-db
 :tables
 (fn [db [_ tables]]
   (assoc db :tables tables)))

(defn get-me [db]
  (if-let [peer-id (-> db :peer-id)]
    (get (get db :persons {}) peer-id)))

(defn determine-active-panel [db]
  (if-let [me (get-me db)]
    (if-let [_ (:table_id me)]
      :table-panel
      (if-let [_ (:pub_id me)]
        :pub-panel
        :home-panel))
    :home-panel))

(defn set-active-panel [db]
  (let [new-panel (determine-active-panel db)]
    (if (not= new-panel (:active-panel db))
      (rf/dispatch [:set-active-panel new-panel]))))

(ti/reg-event-db
 :determine-active-panel
 (fn [db _]
   (set-active-panel db)
   db))

(ti/reg-event-db
 :person
 (fn [db [_ person]]
   (commands/list-pubs (:websocket db))
   (if (and (= (:peer-id db) (:id person)) (-> person :pub_id nil? not))
     (commands/list-tables (:websocket db) (:pub_id person)))
   (let [new-db (assoc-in db [:persons (:id person)] person)]
     (set-active-panel new-db)
     new-db)))

(ti/reg-event-db
 :pub
 (fn [db [_ pub]]
   (assoc-in db [:pubs (:id pub)] pub)))

(ti/reg-event-db
 :table
 (fn [db [_ table]]
   (assoc-in db [:tables (:id table)] table)))

(rf/reg-event-fx
 :ping
 (fn [{:keys [db]} _]
   (if (not (commands/ping (:websocket db)))
     {:dispatch [:create-ws]}
     {})))