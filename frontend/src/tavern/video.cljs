(ns tavern.video
  (:require
   [reagent.core :as reagent]))

(defn video-component [name]
  (let [rtcpeer (reagent/atom nil)
        update
        (fn [comp]
          (let [element (.getElementById js/document name)
                new-argv (reagent/argv comp)
                {:keys [stream type localstream]} (last new-argv)]
            (js/console.log "update video" (str new-argv) name)
            (if (= type :local)
              (set! (.-srcObject element) stream)
              (if (nil? @rtcpeer)
                (let [conn (js/RTCPeerConnection.)
                      tracks (.getTracks localstream)]
                  (doall (map #(.addTrack conn % localstream) tracks))
                  (set! (.-onicecandidate conn) (fn [candidate]
                                                  (js/console.log "candidate" candidate)))
                  (set! (.-onnegotiationneeded conn)
                        (fn []
                          (.then (.createOffer conn)
                                 (fn [offer]
                                   (js/console.log "offer" offer)
                                   (.then (.setLocalDescription conn offer)
                                          (fn []
                                            (js/console.log "local desc" (.-localDescription conn))))))))
                  (set! (.-ontrack conn) (fn [event]
                                           (js/console.log "ontrack" event)))
                  (reset! rtcpeer conn))))))]

    (reagent/create-class
     {:reagent-render (fn [] [:video {:id name :autoPlay true}])
      :component-did-mount update
      :component-did-update update
      :display-name "video-component"})))