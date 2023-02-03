(ns tavern.video
  (:require
   [reagent.core :as reagent]
   [re-frame.core :as rf]
   [tavern.commands :as commands]))

(defn media-stream-wrapper []
  (let [ms (reagent/atom nil)
        last-error (reagent/atom nil)]
    (.catch
     (.then (.getUserMedia (-> js/navigator .-mediaDevices) (js-obj "video" true "audio" false "width" 320))
            (fn [stream]
              (js/console.log "got stream" stream)
              (reset! ms stream)))
     (fn [err]
       (js/console.log "usermedia error" err)
       (reset! last-error err)))
    {:stream ms :error last-error}))

(defn handle-msg [peer conn msg]
  (cond
    (nil? msg) (js/console.log "Null video message from" peer)
    (= (.-type msg) "offer")
    (do (.setRemoteDescription @conn msg)
        (.then (.createAnswer @conn)
               (fn [answer]
                 (js/console.log "answer" answer)
                 (.then (.setLocalDescription @conn answer)
                        (fn []
                          (rf/dispatch [:send peer (.stringify js/JSON (.-localDescription @conn))]))))))
    (= (.-type msg) "answer")
    (.setRemoteDescription @conn msg)
    (not (nil? (.-candidate msg))) (.addIceCandidate @conn msg)
    :else (js/console.log "video msg from" peer (.stringify js/JSON msg))))

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
                (let [config (clj->js {"iceServers" [{"urls" "stun:stun.l.google.com:19302"}]})
                      conn (js/RTCPeerConnection. config)
                      tracks (.getTracks localstream)]
                  (doall (map #(.addTrack conn % localstream) tracks))
                  (set! (.-onicecandidate conn)
                        (fn [candidate]
                          (js/console.log "candidate" candidate)
                          (if (-> candidate .-candidate nil? not)
                            (rf/dispatch [:send name (.stringify js/JSON (.-candidate candidate))]))))
                  (set! (.-onnegotiationneeded conn)
                        (fn []
                          (.then (.createOffer conn)
                                 (fn [offer]
                                   (js/console.log "offer" offer)
                                   (.then (.setLocalDescription conn offer)
                                          (fn []
                                            (js/console.log "local desc" (.-localDescription conn))
                                            (rf/dispatch [:send name (.stringify js/JSON (.-localDescription conn))])))))))
                  (set! (.-ontrack conn)
                        (fn [event]
                          (let [remoteStream (aget (.-streams event) 0)]
                            (js/console.log "ontrack" event remoteStream)
                            (set! (.-srcObject element) remoteStream))))
                  (reset! rtcpeer conn)
                  (rf/dispatch [:peer name rtcpeer]))))))]

    (reagent/create-class
     {:reagent-render (fn [] [:video {:id name :autoPlay true}])
      :component-did-mount update
      :component-did-update update
      :component-will-unmount (fn []
                                (js/console.log "unmounting " name)
                                (if-let [peer @rtcpeer]
                                  (do
                                    (reset! rtcpeer nil)
                                    (.close peer))))
      :display-name "video-component"})))
