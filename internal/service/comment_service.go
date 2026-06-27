package service

import (
	"gorm.io/gorm"

	"minibili/internal/model"
)

// CommentService encapsulates comment-domain business rules.
type CommentService struct {
	DB *gorm.DB
}

// ListByVideo returns top-level comments for a video, with optional sort.
func (s *CommentService) ListByVideo(vid uint64, sortKey string) ([]model.Comment, error) {
	orderClause := "id ASC"
	switch sortKey {
	case "hot":
		orderClause = "like_count DESC, id DESC"
	case "latest":
		orderClause = "created_at DESC, id DESC"
	}
	var list []model.Comment
	if err := s.DB.Where("video_id = ? AND parent_id = 0", vid).Order(orderClause).Find(&list).Error; err != nil {
		return nil, err
	}
	return list, nil
}

// ListReplies returns child comments for a given parent.
func (s *CommentService) ListReplies(parentID uint64) ([]model.Comment, error) {
	var list []model.Comment
	if err := s.DB.Where("parent_id = ?", parentID).Order("id ASC").Find(&list).Error; err != nil {
		return nil, err
	}
	return list, nil
}

// Create inserts a new comment.
func (s *CommentService) Create(c *model.Comment) error {
	return s.DB.Create(c).Error
}

// Delete removes a comment and all its nested replies.
func (s *CommentService) Delete(id uint64) error {
	// cascade delete nested replies
	s.DB.Where("parent_id = ?", id).Delete(&model.Comment{})
	return s.DB.Delete(&model.Comment{}, id).Error
}

// ToggleLike flips a user's like on a comment.
func (s *CommentService) ToggleLike(commentID, userID uint64) (added bool, err error) {
	var existing model.CommentLike
	err = s.DB.Where("comment_id = ? AND user_id = ?", commentID, userID).First(&existing).Error
	if err == nil {
		// Unlike
		s.DB.Delete(&existing)
		s.DB.Model(&model.Comment{}).Where("id = ?", commentID).
			UpdateColumn("like_count", gorm.Expr("GREATEST(like_count - 1, 0)"))
		return false, nil
	}
	// Like
	like := model.CommentLike{CommentID: commentID, UserID: userID}
	if err := s.DB.Create(&like).Error; err != nil {
		return false, err
	}
	s.DB.Model(&model.Comment{}).Where("id = ?", commentID).
		UpdateColumn("like_count", gorm.Expr("like_count + 1"))
	return true, nil
}
