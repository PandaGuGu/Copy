package data

import (
	"go.uber.org/zap"
	"golang.org/x/crypto/bcrypt"
	"gorm.io/gorm"

	"minibili/internal/model"
)

const SystemUsername = "cakecake_system"

// EnsureSystemUser creates the system notification user if it does not exist.
// Returns the user ID for use in system-sent notifications.
func EnsureSystemUser(db *gorm.DB, lg *zap.Logger) (uint64, error) {
	var u model.User
	err := db.Where("username = ?", SystemUsername).First(&u).Error
	if err == nil {
		return u.ID, nil
	}
	if err != gorm.ErrRecordNotFound {
		return 0, err
	}

	// Create system user
	hash, _ := bcrypt.GenerateFromPassword([]byte(""), bcrypt.DefaultCost)
	sys := model.User{
		Username:    SystemUsername,
		Nickname:    "系统通知",
		PasswordHash: string(hash), // bcrypt of empty string, no valid password
		Status:      "active",
		Sign:        "cakecake 官方系统通知账号",
	}
	if err := db.Create(&sys).Error; err != nil {
		return 0, err
	}

	// Set CakeID now that we have the ID
	_ = db.Model(&sys).Update("cake_id", model.FormatCakeID(sys.ID)).Error

	if lg != nil {
		lg.Info("seed system user created",
			zap.String("username", SystemUsername),
			zap.Uint64("user_id", sys.ID),
		)
	}
	return sys.ID, nil
}
