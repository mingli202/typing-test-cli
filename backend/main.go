package main

import (
	"context"
	"encoding/json"
	"fmt"
	"log"
	"net"
	"net/http"
	"os"
	"os/signal"
	"syscall"
	"time"
	"tui/backend/handlers/hub"
	"tui/backend/services/data_provider"
	"tui/backend/services/name_provider"
)

const port = 8080

func main() {
	mux := http.NewServeMux()

	err := registerRoutes(mux)

	if err != nil {
		log.Fatal(err)
	}

	ctx := context.Background()

	server := &http.Server{
		Addr:    fmt.Sprintf(":%v", port),
		Handler: mux,
		BaseContext: func(l net.Listener) context.Context {
			ctx = context.WithValue(ctx, "serverAddr", l.Addr().String())
			return ctx
		},
	}

	go func() {
		log.Printf("Starting server on port %v\n", port)
		err := server.ListenAndServe()

		if err != nil {
			log.Println(err)
		}
	}()

	quit := make(chan os.Signal, 1)
	signal.Notify(quit, syscall.SIGINT, syscall.SIGTERM)

	<-quit
	log.Println("Shutting down server")

	ctx, cancel := context.WithTimeout(context.Background(), time.Second*10)
	defer cancel()

	if err := server.Shutdown(ctx); err != nil {
		log.Printf("Server forced to shut down, %s\n", err)
	}

	log.Println("Server stopped")
}

func registerRoutes(mux *http.ServeMux) error {
	dataProvider, err := data_provider.NewDataProvider()

	if err != nil {
		return err
	}

	nameProvider, err := name_provider.NewNameProvider()

	if err != nil {
		return err
	}

	mux.HandleFunc("/health", func(w http.ResponseWriter, r *http.Request) {
		ctx := r.Context()
		fmt.Fprintf(w, "Ready at %v!\n", ctx.Value("serverAddr"))
	})
	mux.Handle("/ws", hub.Handler(&dataProvider, &nameProvider))
	mux.HandleFunc("/new_data", func(w http.ResponseWriter, r *http.Request) {
		data, err := dataProvider.NewData()

		if err != nil {
			return
		}

		p, err := json.Marshal(data)

		if err != nil {
			return
		}

		w.Write(p)
	})

	return nil
}
