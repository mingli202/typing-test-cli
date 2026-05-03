package models

type ToMsg interface {
	ToMsg() (string, error)
}
