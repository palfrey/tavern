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
   :persons
   (fn [db _]
     (get db :persons [])))

  (rf/reg-sub
   :pubs
   (fn [db _]
     (get db :pubs [])))

  (rf/reg-sub
   :me
   (fn [_]
     [(rf/subscribe [:peer-id]) (rf/subscribe [:persons])])

   (fn [[peer-id persons] _]
     (get persons peer-id)))

  (rf/reg-sub
   :current-pub
   (fn [_]
     [(rf/subscribe [:me]) (rf/subscribe [:pubs])])

   (fn [[me pubs] _]
     (if-let [pub_id (:pub_id me)]
       (get pubs pub_id)
       {})))

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
  (let [pubName (reagent/atom "")]
    (fn []
      [:div [:input {:type "button" :value "Update pub list" :onClick #(commands/list-pubs @(rf/subscribe [:websocket]))}]
       [:div "Pubs"]
       [:ul
        (for [pub (vals @(rf/subscribe [:pubs]))]
          ^{:key (:id pub)}
          [:li (:name pub)
           [:span " "]
           [:button {:class "btn btn-primary" :onClick #(commands/join-pub @(rf/subscribe [:websocket]) (:id pub))} "Join"]
           [:span " "]
           (if (= (:persons pub) [])
             [:button {:class "btn btn-danger"
                       :onClick #(commands/delete-pub @(rf/subscribe [:websocket]) (:id pub))} "Delete"] [:div])])]
       [:form
        [:div {:class "form-group"}
         [:label {:for "pubName"} "New pub"]
         [:input {:type "text"
                  :class "form-control"
                  :id "pubName"
                  :placeholder "Enter pub name"
                  :value @pubName
                  :on-change (fn [evt]
                               (reset! pubName (-> evt .-target .-value)))}]]
        [:button {:type "button" :class "btn btn-primary" :onClick #(commands/create-pub @(rf/subscribe [:websocket]) @pubName)} "Create pub"]]
       [videos]])))

(defn about-panel []
  [:div "This is the About Page."])

(defn pub-panel []
  (let [current_pub @(rf/subscribe [:current-pub])]
    [:div [:span (str "Pub " (:name current_pub))]
     [:button {:class "btn btn-danger"
               :onClick #(commands/leave-pub @(rf/subscribe [:websocket]))} "Leave pub"]]))

(defn- panels [panel-name]
  (case panel-name
    :home-panel [home-panel]
    :about-panel [about-panel]
    :pub-panel [pub-panel]
    [:div (str "Missing panel " panel-name)]))

(defn show-panel [panel-name]
  [panels panel-name])

(defn main-panel []
  (let [active-panel (rf/subscribe [:active-panel])]
    [show-panel @active-panel]))

(defn ui []
  [:div
   [:nav {:class "navbar navbar-expand-md navbar-dark bg-dark fixed-top"}
    [:a {:class "navbar-brand", :href "#"} "Tavern"]
    [:button {:class "navbar-toggler", :type "button", :data-toggle "collapse", :data-target "#navbarsExampleDefault", :aria-controls "navbarsExampleDefault", :aria-expanded "false", :aria-label "Toggle navigation"}
     [:span {:class "navbar-toggler-icon"}]]
    [:div {:class "collapse navbar-collapse", :id "navbarsExampleDefault"}
     [:ul {:class "navbar-nav mr-auto"}
      [:li {:class "nav-item active"}
       [:a {:class "nav-link", :href "#"} "Home "
        [:span {:class "sr-only"} "(current)"]]]
      [:li {:class "nav-item"}
       [:a {:class "nav-link", :href "#"} "Link"]]]
     [:span {:class "navbar-text"} [clock]]]]
   [:main {:role "main" :class "container-fluid"}
    [:h1 "Tavern"]
    [main-panel]]])

(defn render []
  (rdom/render [ui]
               (js/document.getElementById "root")))

(defn ^:dev/before-load stop []
  (js/console.log "stop"))

(defn ^:dev/after-load start []
  (js/console.log "start")
  (rf/clear-subscription-cache!)
  (ti/interval-handler {:action :clean})
  (reg-subs)
  (rf/dispatch [:determine-active-panel])
  (render))

(defn ^:export init []
  (js/console.log "init")
  (rf/dispatch-sync [:create-ws])
  ;(events/getMediaStream)
  (start))
