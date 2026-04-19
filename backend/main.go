package main

import (
	"fmt"
	"log"
	"net/http"
)

type GameHandler struct{}

func (GameHandler) ServeHTTP(http.ResponseWriter, *http.Request) {}

func main() {
	mux := http.NewServeMux()

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		fmt.Fprintf(w, "Ready!")
	})

	log.Fatal(http.ListenAndServe(":8080", mux))
}
