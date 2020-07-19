(ns tavern.video
  (:require
   [reagent.core :as reagent]))

(defn video-component [name]
  (let [update
        (fn [comp]
          (let [element (.getElementById js/document name)
                new-argv (reagent/argv comp)
                {:keys [stream]} (last new-argv)]
            (set! (.-srcObject element) stream)))]

    (reagent/create-class
     {:reagent-render (fn [] [:video {:id name :autoPlay true}])
      :component-did-mount update
      :component-did-update update
      :display-name "video-component"})))