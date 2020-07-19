(ns fmt
  (:require
   [hawk.core :as hawk]
   [cljfmt.main]
   [clojure.string :as str]))

; https://clojureverse.org/t/every-pred-for-functions-with-multiple-arguments/5423/2
(defn every-preds [& preds]
  (fn [& args]
    ((apply every-pred
            (mapv
             (fn [p]
               (fn [args]
                 (apply p args)))
             preds))
     args)))

(hawk/watch!
 [{:paths ["."]
   :filter (every-preds
            hawk/file?
            (fn [ctx {:keys [file kind]}]
              (not-any?
               #(str/starts-with? % ".")
               (str/split (.getPath file) (re-pattern java.io.File/separator)))))
   :handler
   (fn [ctx {:keys [file kind]}]
     (let [path (.getAbsolutePath file)
           extension (last (str/split path #"\."))]
       (if (contains? #{"clj" "cljs" "edn"} extension)
         (cljfmt.main/fix [file] {}))))}])