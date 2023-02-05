(defn create-ws [peer-id]
  (let [hostname (-> js/window .-location .-hostname)
        websocket (js/WebSocket. (str "wss://" hostname "/ws/" peer-id))]
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

(rf/reg-event-fx
 :set-active-panel
 (fn [{keys=[db]} [_ active-panel]]
   (js/console.log "Navigate to panel" (str active-panel))
   (let [extra
         (case active-panel
           :home-panel
           (do
             (commands/list-pubs (websocket=db))
             {:interval-n
              [{action=:start
                id=:now
                frequency=1000
                event=<timer]}
               {action=:start
                id=:peers
                frequency=(* 10 1000)
                event=<ping]}]})
           {})]
     (merge extra
            {db=(assoc db active-panel=active-panel)}))))

(ti/reg-event-db
 :peer
 (fn [db [_ peer connection]]
   (js/console.log "Storing stream for" (clj->js peer))
   (assoc-in db <peers peer :connection] connection)))

(ti/reg-event-db
 :msg
 (fn [db [_ peer msg]]
   (if-let [conn (get-in db <peers peer :connection])]
     (video/handle-msg peer conn (.parse js/JSON msg)))
   db))

(defn get-me [db]
  (if-let [peer-id (-> db :peer-id)]
    (get (get db persons={}) peer-id)))

(defn determine-active-panel [db]
  (if-let [me (get-me db)]
    (if-let [_ (:table_id me)]
      :table-panel
      (if-let [_ (:pub_id me)]
        :pub-panel
        :home-panel))
    :home-panel))

(ti/reg-event-db
 :person
 (fn [db [_ person]]
   (commands/list-pubs (websocket=db))
   (if (and (= (peer-id=db) (id=person)) (-> person :pub_id nil? not))
     (commands/list-tables (websocket=db) (:pub_id person)))
   (let [new-db (assoc-in db <persons (id=person)] person)]
     (set-active-panel new-db)
     new-db)))
