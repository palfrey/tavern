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
     (get db :pubs {})))

  (rf/reg-sub
   :tables
   (fn [db _]
     (get db :tables {})))

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
   :current-table
   (fn [_]
     [(rf/subscribe [:me]) (rf/subscribe [:tables])])

   (fn [[me tables] _]
     (if-let [table_id (:table_id me)]
       (get tables table_id)
       {})))

  (rf/reg-sub
   :websocket
   (fn [db _]
     (:websocket db)))

  (rf/reg-sub
   :mediastream
   (fn [db _]
     (if-let [ms (get db :mediastream)]
       @(:stream ms)
       nil))))

(defn clock []
  [:div.example-clock
   (-> @(rf/subscribe [:time])
       .toTimeString
       (str/split " ")
       first)])

(defn getstreams []
  (let [peerid @(rf/subscribe [:peer-id])
        stream @(rf/subscribe [:mediastream])
        peers (get @(rf/subscribe [:current-table]) :persons [])
        peers (filter #(not= % peerid) peers)]
    (if (or (nil? peerid) (nil? stream))
      []
      (do
        (cons ^{:key peerid} [video/video-component peerid {:type :local :stream stream}]
              (map #(with-meta (vector video/video-component % {:type :remote :localstream stream}) {:key %}) peers))))))

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
           (let [entry (nth streams idx)]
             ^{:key (gstring/format "stream-%d" idx)}
             [:td {:style {:border 1 :border-style "solid" :border-color "black"} :width (gstring/format "%f%%" (/ 100 size))}
              [:div {:style {:color "white"}} (gstring/format "stream-%d" idx)]
              entry]))])]]))

(defn home-panel []
  (let [pubName (reagent/atom "")]
    (fn []
      [:div
       [:h1 "Tavern"]
       [:input {:type "button" :class "btn btn-secondary" :value "Update pub list" :onClick #(commands/list-pubs @(rf/subscribe [:websocket]))}]
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
        [:button {:type "button" :class "btn btn-primary" :onClick #(commands/create-pub @(rf/subscribe [:websocket]) @pubName)} "Create pub"]]])))

(defn about-panel []
  [:div "This is the About Page."])

(defn pub-panel []
  (let [tableName (reagent/atom "")]
    (fn []
      (let [current_pub @(rf/subscribe [:current-pub])]
        [:div [:h1 (:name current_pub)]
         [:br]
         [:button {:class "btn btn-danger"
                   :onClick #(commands/leave-pub @(rf/subscribe [:websocket]))} "Leave pub"]
         [:br]
         [:input {:type "button" :class "btn btn-secondary" :value "Update table list" :onClick #(commands/list-tables @(rf/subscribe [:websocket]) (:id current_pub))}]
         [:div "Tables"]
         [:ul
          (for [table (vals @(rf/subscribe [:tables]))]
            ^{:key (:id table)}
            [:li (:name table)
             [:span " "]
             [:button {:class "btn btn-primary" :onClick #(commands/join-table @(rf/subscribe [:websocket]) (:id table))} "Join"]
             [:span " "]
             (if (= (:persons table) [])
               [:button {:class "btn btn-danger"
                         :onClick #(commands/delete-table @(rf/subscribe [:websocket]) (:id table))} "Delete"] [:div])])]
         [:form
          [:div {:class "form-group"}
           [:label {:for "tableName"} "New table"]
           [:input {:type "text"
                    :class "form-control"
                    :id "tableName"
                    :placeholder "Enter table name"
                    :value @tableName
                    :on-change (fn [evt]
                                 (reset! tableName (-> evt .-target .-value)))}]]
          [:button {:type "button" :class "btn btn-primary" :onClick #(commands/create-table @(rf/subscribe [:websocket]) (:id current_pub) @tableName)} "Create table"]]]))))

(defn table-panel []
  (let [current_pub @(rf/subscribe [:current-pub])
        current_table @(rf/subscribe [:current-table])]
    [:div
     [:h1 (gstring/format "%s: %s" (:name current_pub) (:name current_table))]
     [:br]
     [:button {:class "btn btn-danger"
               :onClick #(commands/leave-table @(rf/subscribe [:websocket]))} "Leave table"]
     [videos]]))

(defn- panels [panel-name]
  (case panel-name
    :home-panel [home-panel]
    :about-panel [about-panel]
    :pub-panel [pub-panel]
    :table-panel [table-panel]
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
  (rf/dispatch [:mediastream (video/media-stream-wrapper)])
  (rf/dispatch [:determine-active-panel])
  (render))

(defn ^:export init []
  (js/console.log "init")
  (rf/dispatch-sync [:create-ws])
  (start))
