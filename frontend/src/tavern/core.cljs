(ns tavern.core
  (:require
   ["webrtc-adapter" :as webrtc-adapter]
   [reagent.core :as reagent]
   [reagent.dom :as rdom]
   [re-frame.core :as rf]
   [clojure.string :as str]
   [goog.string :as gstring]
   [goog.string.format]

   [tavern.commands :as commands]
   [tavern.routes :as routes]
   [tavern.intervals :as ti]
   [tavern.events :as events]
   [tavern.video :as video]))

(set! *warn-on-infer* true)

(defn reg-subs []
  (rf/clear-sub)

  (rf/reg-sub
   :time
   (fn [db _]
     (get db :time (js/Date.))))

  (rf/reg-sub
   :active-panel
   (fn [db _]
     (:active-panel db)))

  (rf/reg-sub
   :peer-id
   (fn [db _]
     (get db :peer-id nil)))

  (rf/reg-sub
   :peers
   (fn [db _]
     (get db :peers [])))

  (rf/reg-sub
   :websocket
   (fn [db _]
     (:websocket db)))

  (rf/reg-sub
   :mediastream
   (fn [db _]
     (get db :mediastream nil))))

(defn clock []
  [:div.example-clock
   (-> @(rf/subscribe [:time])
       .toTimeString
       (str/split " ")
       first)])

(defn getstreams []
  (let [peerid @(rf/subscribe [:peer-id])
        stream @(rf/subscribe [:mediastream])]
    (if (or (nil? peerid) (nil? stream))
      []
      (do
        (cons [peerid {:stream stream}]
              (seq @(rf/subscribe [:peers])))))))

(defn videos []
  (let [streams (getstreams)
        total (count streams)
        size (Math/ceil (if (> total 0) (Math/sqrt total) 0))]
    (js/console.log "size" size)
    (js/console.log "streams" (clj->js streams))
    [:table {:width "100%" :style {:background-color "black" :border 1 :border-style "solid" :border-color "black"}}
     [:tbody {:width "100%"}
      (for [x (range size)]
        ^{:key (gstring/format "row-%d" x)}
        [:tr {:width "100%"}
         (for [y (range size)
               :let [idx (+ y (* x size))]
               :when (< idx total)]
           (let [entry (nth streams idx)
                 id (first entry)
                 config (second entry)]
             ^{:key (gstring/format "stream-%d" idx)}
             [:td {:style {:border 1 :border-style "solid" :border-color "black"} :width (gstring/format "%f%%" (/ 100 size))}
              [:div {:style {:color "white"}} (gstring/format "stream-%d" idx)]
              [video/video-component (gstring/format "stream-%d" idx) config]]))])]]))

(defn home-panel []
  [:div [:input {:type "button" :value "Update pub list" :onClick #(commands/list-pubs @(rf/subscribe [:websocket]))}]
   [:div @(rf/subscribe [:peer-id])]
   [:div (str @(rf/subscribe [:peers]))]
   [:div [:a {:href (routes/url-for :about)} "go to About Page"]]
   [videos]])

(defn about-panel []
  [:div "This is the About Page."
   [:div [:a {:href (routes/url-for :home)} "go to Home Page"]]])

(defn- panels [panel-name]
  (case panel-name
    :home-panel [home-panel]
    :about-panel [about-panel]
    [:div]))

(defn show-panel [panel-name]
  [panels panel-name])

(defn main-panel []
  (let [active-panel (rf/subscribe [:active-panel])]
    [show-panel @active-panel]))

(defn ui []
  [:div
   [:h1 "Tavern"]
   [clock]
   [main-panel]])

(defn render []
  (rdom/render [ui]
               (js/document.getElementById "root")))

(defn ^:dev/before-load stop []
  (js/console.log "stop"))

(defn ^:dev/after-load start []
  (js/console.log "start")
  (routes/app-routes)
  (rf/clear-subscription-cache!)
  (ti/interval-handler {:action :clean})
  (reg-subs)
  (render))

(defn ^:export init []
  (js/console.log "init")
  (rf/dispatch-sync [:initialize])
  ;(events/getMediaStream)
  (start))
