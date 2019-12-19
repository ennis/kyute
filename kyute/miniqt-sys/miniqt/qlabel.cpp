#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QLabel>
#include <iostream>

QLabel *QLabel_new(QWidget *parent) { return new QLabel{parent}; }

QWidget *QLabel_upcast(QLabel *this_) { return this_; }

void QLabel_setText(QLabel *this_, MQStringRef text) {
  this_->setText(makeQString(text));
}

void QLabel_setTextFormat(QLabel *this_, Qt_TextFormat format) {
  this_->setTextFormat(static_cast<Qt::TextFormat>(format));
}

void QLabel_delete(QLabel *this_) {
  std::cerr << "QPushButton_delete(" << this_ << ")\n";
  delete this_;
}