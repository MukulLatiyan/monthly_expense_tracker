package main

import (
	"expense-tracker/db"
	"expense-tracker/handlers"
	"log"
	"os"
	"strings"

	"github.com/aws/aws-lambda-go/events"
	"github.com/aws/aws-lambda-go/lambda"
	fiberadapter "github.com/awslabs/aws-lambda-go-api-proxy/fiber"
	"github.com/gofiber/fiber/v2"
	"github.com/gofiber/fiber/v2/middleware/cors"
	"github.com/gofiber/fiber/v2/middleware/recover"
)

var fiberLambda *fiberadapter.FiberLambda

func init() {
	log.Printf("Initializing Lambda function")

	app := fiber.New(fiber.Config{
		DisableStartupMessage: true,
	})

	app.Use(recover.New())
	app.Use(cors.New(cors.Config{
		AllowOrigins:  "*",
		AllowMethods:  "GET, POST, PUT, DELETE, OPTIONS",
		AllowHeaders:  "*",
		ExposeHeaders: "Content-Type",
	}))

	repo, err := db.NewDynamoDBRepository()
	if err != nil {
		log.Printf("Error initializing DynamoDB repository: %v", err)
		os.Exit(1)
	}

	h := handlers.NewHandler(repo)
	h.SetupRoutes(app)

	fiberLambda = fiberadapter.New(app)
	log.Printf("Lambda initialized successfully")
}

func stripStagePrefix(path string) string {
	if path == "" {
		return "/"
	}
	// Remove leading slash if present
	path = strings.TrimPrefix(path, "/")
	
	// Find the next slash (after the stage name)
	idx := strings.Index(path, "/")
	if idx == -1 {
		// No more path after stage, return root
		return "/"
	}
	
	// Return everything after the stage name
	return "/" + path[idx+1:]
}

func Handler(request events.APIGatewayV2HTTPRequest) (events.APIGatewayV2HTTPResponse, error) {
	// HTTP API v2 uses RawPath or RequestContext.HTTP.Path
	originalPath := request.RawPath
	if originalPath == "" {
		originalPath = request.RequestContext.HTTP.Path
	}
	if originalPath == "" {
		originalPath = "/"
	}
	
	rewrittenPath := stripStagePrefix(originalPath)
	
	// Log all request details for debugging
	log.Printf("Request: method=%s path=%s rawPath=%s rewritten=%s", 
		request.RequestContext.HTTP.Method,
		request.RequestContext.HTTP.Path,
		request.RawPath,
		rewrittenPath)
	
	// Create a modified request
	modifiedRequest := request
	modifiedRequest.RawPath = rewrittenPath
	modifiedRequest.RequestContext.HTTP.Path = rewrittenPath
	
	return fiberLambda.ProxyV2(modifiedRequest)
}

func main() {
	lambda.Start(Handler)
}
