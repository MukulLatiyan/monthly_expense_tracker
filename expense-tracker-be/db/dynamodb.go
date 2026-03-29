package db

import (
	"context"
	"fmt"
	"os"

	"expense-tracker/models"

	"github.com/aws/aws-sdk-go-v2/aws"
	"github.com/aws/aws-sdk-go-v2/config"
	"github.com/aws/aws-sdk-go-v2/feature/dynamodb/attributevalue"
	"github.com/aws/aws-sdk-go-v2/service/dynamodb"
	"github.com/aws/aws-sdk-go-v2/service/dynamodb/types"
)

type DynamoDBRepository struct {
	Client    *dynamodb.Client
	TableName string
}

func NewDynamoDBRepository() (*DynamoDBRepository, error) {
	ctx := context.Background()

	region := os.Getenv("AWS_REGION")
	if region == "" {
		region = "ap-south-1"
	}

	cfg, err := config.LoadDefaultConfig(ctx, config.WithRegion(region))
	if err != nil {
		return nil, fmt.Errorf("unable to load SDK config: %v", err)
	}

	tableName := os.Getenv("DYNAMODB_TABLE")
	if tableName == "" {
		tableName = "expense_tracker"
	}

	return &DynamoDBRepository{
		Client:    dynamodb.NewFromConfig(cfg),
		TableName: tableName,
	}, nil
}

func (r *DynamoDBRepository) GetExpenses(ctx context.Context, month string) ([]models.Expense, error) {
	input := &dynamodb.QueryInput{
		TableName:              aws.String(r.TableName),
		KeyConditionExpression: aws.String("pk = :pk AND begins_with(sk, :sk_prefix)"),
		ExpressionAttributeValues: map[string]types.AttributeValue{
			":pk":        &types.AttributeValueMemberS{Value: "MONTH#" + month},
			":sk_prefix": &types.AttributeValueMemberS{Value: "EXPENSE#"},
		},
	}

	result, err := r.Client.Query(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("failed to query expenses: %v", err)
	}

	var expenses []models.Expense
	for _, item := range result.Items {
		var exp models.Expense
		err := attributevalue.UnmarshalMap(item, &exp)
		if err != nil {
			continue
		}
		expenses = append(expenses, exp)
	}

	return expenses, nil
}

func (r *DynamoDBRepository) GetExpense(ctx context.Context, month, name string) (*models.Expense, error) {
	input := &dynamodb.GetItemInput{
		TableName: aws.String(r.TableName),
		Key: map[string]types.AttributeValue{
			"pk": &types.AttributeValueMemberS{Value: "MONTH#" + month},
			"sk": &types.AttributeValueMemberS{Value: "EXPENSE#" + name},
		},
	}

	result, err := r.Client.GetItem(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("failed to get expense: %v", err)
	}

	if result.Item == nil {
		return nil, nil
	}

	var exp models.Expense
	err = attributevalue.UnmarshalMap(result.Item, &exp)
	if err != nil {
		return nil, err
	}

	return &exp, nil
}

func (r *DynamoDBRepository) AddExpense(ctx context.Context, expense models.Expense) error {
	item, err := attributevalue.MarshalMap(expense)
	if err != nil {
		return fmt.Errorf("failed to marshal expense: %v", err)
	}

	item["pk"] = &types.AttributeValueMemberS{Value: "MONTH#" + expense.Month}
	item["sk"] = &types.AttributeValueMemberS{Value: "EXPENSE#" + expense.Name}

	input := &dynamodb.PutItemInput{
		TableName:           aws.String(r.TableName),
		Item:                item,
		ConditionExpression: aws.String("attribute_not_exists(pk) AND attribute_not_exists(sk)"),
	}

	_, err = r.Client.PutItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to add expense: %v", err)
	}

	return nil
}

func (r *DynamoDBRepository) UpdateExpense(ctx context.Context, expense models.Expense) error {
	item, err := attributevalue.MarshalMap(expense)
	if err != nil {
		return fmt.Errorf("failed to marshal expense: %v", err)
	}

	item["pk"] = &types.AttributeValueMemberS{Value: "MONTH#" + expense.Month}
	item["sk"] = &types.AttributeValueMemberS{Value: "EXPENSE#" + expense.Name}

	input := &dynamodb.PutItemInput{
		TableName: aws.String(r.TableName),
		Item:      item,
	}

	_, err = r.Client.PutItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to update expense: %v", err)
	}

	return nil
}

func (r *DynamoDBRepository) DeleteExpense(ctx context.Context, month, name string) error {
	input := &dynamodb.DeleteItemInput{
		TableName: aws.String(r.TableName),
		Key: map[string]types.AttributeValue{
			"pk": &types.AttributeValueMemberS{Value: "MONTH#" + month},
			"sk": &types.AttributeValueMemberS{Value: "EXPENSE#" + name},
		},
	}

	_, err := r.Client.DeleteItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to delete expense: %v", err)
	}

	return nil
}

func (r *DynamoDBRepository) GetIncome(ctx context.Context, month string) ([]models.Income, error) {
	input := &dynamodb.QueryInput{
		TableName:              aws.String(r.TableName),
		KeyConditionExpression: aws.String("pk = :pk AND begins_with(sk, :sk_prefix)"),
		ExpressionAttributeValues: map[string]types.AttributeValue{
			":pk":        &types.AttributeValueMemberS{Value: "MONTH#" + month},
			":sk_prefix": &types.AttributeValueMemberS{Value: "INCOME#"},
		},
	}

	result, err := r.Client.Query(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("failed to query income: %v", err)
	}

	var income []models.Income
	for _, item := range result.Items {
		var inc models.Income
		err := attributevalue.UnmarshalMap(item, &inc)
		if err != nil {
			continue
		}
		income = append(income, inc)
	}

	return income, nil
}

func (r *DynamoDBRepository) GetIncomeItem(ctx context.Context, month, name string) (*models.Income, error) {
	input := &dynamodb.GetItemInput{
		TableName: aws.String(r.TableName),
		Key: map[string]types.AttributeValue{
			"pk": &types.AttributeValueMemberS{Value: "MONTH#" + month},
			"sk": &types.AttributeValueMemberS{Value: "INCOME#" + name},
		},
	}

	result, err := r.Client.GetItem(ctx, input)
	if err != nil {
		return nil, fmt.Errorf("failed to get income: %v", err)
	}

	if result.Item == nil {
		return nil, nil
	}

	var inc models.Income
	err = attributevalue.UnmarshalMap(result.Item, &inc)
	if err != nil {
		return nil, err
	}

	return &inc, nil
}

func (r *DynamoDBRepository) AddIncome(ctx context.Context, income models.Income) error {
	item, err := attributevalue.MarshalMap(income)
	if err != nil {
		return fmt.Errorf("failed to marshal income: %v", err)
	}

	item["pk"] = &types.AttributeValueMemberS{Value: "MONTH#" + income.Month}
	item["sk"] = &types.AttributeValueMemberS{Value: "INCOME#" + income.Name}

	input := &dynamodb.PutItemInput{
		TableName:           aws.String(r.TableName),
		Item:                item,
		ConditionExpression: aws.String("attribute_not_exists(pk) AND attribute_not_exists(sk)"),
	}

	_, err = r.Client.PutItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to add income: %v", err)
	}

	return nil
}

func (r *DynamoDBRepository) UpdateIncome(ctx context.Context, income models.Income) error {
	item, err := attributevalue.MarshalMap(income)
	if err != nil {
		return fmt.Errorf("failed to marshal income: %v", err)
	}

	item["pk"] = &types.AttributeValueMemberS{Value: "MONTH#" + income.Month}
	item["sk"] = &types.AttributeValueMemberS{Value: "INCOME#" + income.Name}

	input := &dynamodb.PutItemInput{
		TableName: aws.String(r.TableName),
		Item:      item,
	}

	_, err = r.Client.PutItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to update income: %v", err)
	}

	return nil
}

func (r *DynamoDBRepository) DeleteIncome(ctx context.Context, month, name string) error {
	input := &dynamodb.DeleteItemInput{
		TableName: aws.String(r.TableName),
		Key: map[string]types.AttributeValue{
			"pk": &types.AttributeValueMemberS{Value: "MONTH#" + month},
			"sk": &types.AttributeValueMemberS{Value: "INCOME#" + name},
		},
	}

	_, err := r.Client.DeleteItem(ctx, input)
	if err != nil {
		return fmt.Errorf("failed to delete income: %v", err)
	}

	return nil
}
