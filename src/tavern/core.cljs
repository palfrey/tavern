(ns tavern.core
  (:require
   ["peerjs" :as peerjs]
   [reagent.core :as reagent]
   [reagent.dom :as rdom]
   [re-frame.core :as rf]
   [clojure.string :as str]

   [tavern.routes :as routes]
   [tavern.intervals :as ti]
   [tavern.events :as events]))

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
     (get db :peers []))))

(defn clock []
  [:div.example-clock
   (-> @(rf/subscribe [:time])
       .toTimeString
       (str/split " ")
       first)])

(defn home-panel []
  [:div (str "This is the Home Page.")
   [:div @(rf/subscribe [:peer-id])]
   [:div (str @(rf/subscribe [:peers]))]
   [:div [:a {:href (routes/url-for :about)} "go to About Page"]]])

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
  (events/getMediaStream)
  (render))

(defn ^:export init []
  (js/console.log "init")
  (rf/dispatch-sync [:initialize])
  (start))
