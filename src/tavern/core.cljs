(ns tavern.core
  (:require ["peerjs" :as peerjs]
            [reagent.core :as reagent]
            [reagent.dom :as rdom]
            [re-frame.core :as rf]
            [tavern.routes :as routes]
            [clojure.string :as str]))

(set! *warn-on-infer* true)

(rf/reg-event-db
 :initialize
 (fn [_ _]
   {:time (js/Date.)}))

(defn dispatch-timer-event []
  (let [now (js/Date.)]
    (rf/dispatch [:timer now])))

(defonce do-timer
  (js/setInterval dispatch-timer-event 1000))

(rf/reg-event-db
 :timer
 (fn [db [_ new-time]]
   (assoc db :time new-time)))

(rf/reg-sub
 :time
 (fn [db _]
   (:time db)))

(rf/reg-sub
 ::active-panel
 (fn [db _]
   (:active-panel db)))

(rf/reg-event-db
 :timer
 (fn [db [_ new-time]]
   (assoc db :time new-time)))

(defn clock []
  [:div.example-clock
   (-> @(rf/subscribe [:time])
       .toTimeString
       (str/split " ")
       first)])

(defn home-panel []
  [:div (str "This is the Home Page.")
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
  (let [active-panel (rf/subscribe [::active-panel])]
    [show-panel @active-panel]))

(defn ui []
  [:div
   [:h1 "Hello world, it is now"]
   [clock]
   [main-panel]])

(defn render []
  (rdom/render [ui]
               (js/document.getElementById "root")))

(defn ^:export init []
  (js/console.log "init")
  (routes/app-routes)
  (rf/dispatch-sync [:initialize])
  (render))