package main

import (
	"do-manager/manager"
	"flag"
	"log"
	"os"
)

var (
	tokenFlag          = flag.String("token", "", "DigitalOcean read & write access token")
	clusterIDFlag      = flag.String("clusterid", "", "ID of a cluster in DigitalOcean")
	deploymentNameFlag = flag.String("deployment", "", "Name of provers deployment in DigitalOcean k8s cluster")
	serverPortFlag     = flag.Uint("serverPort", 0, "Port number to listen for manage requests")
)

func main() {
	flag.Parse()
	if *tokenFlag == "" || *clusterIDFlag == "" || *deploymentNameFlag == "" || *serverPortFlag == 0 {
		flag.Usage()
		os.Exit(2)
	}

	mg, err := manager.New(*tokenFlag, *clusterIDFlag, *deploymentNameFlag)
	if err != nil {
		log.Fatalf("failed to create a manager: %v", err)
	}

	mg.Add()

	// http.HandleFunc("/add", func(w http.ResponseWriter, r *http.Request) {
	// 	log.Println("received /add request")
	// 	err := mg.AddProver()
	// 	if err != nil {
	// 		log.Printf("failed to add prover: %v", err)
	// 		w.WriteHeader(http.StatusInternalServerError)
	// 		return
	// 	}
	// 	log.Println("/add request processed")
	// })

	// http.HandleFunc("/cut", func(w http.ResponseWriter, r *http.Request) {
	// 	log.Println("received /cut request")
	// 	err := mg.CutProver()
	// 	if err != nil {
	// 		log.Printf("failed to cut prover: %v", err)
	// 		w.WriteHeader(http.StatusInternalServerError)
	// 		return
	// 	}
	// 	log.Println("/cut request processed")
	// })

	// log.Printf("listening at :%d\n", *serverPortFlag)
	// log.Fatal(http.ListenAndServe(fmt.Sprintf(":%d", *serverPortFlag), nil))
}
