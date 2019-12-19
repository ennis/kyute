#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QWidget>
#include <iostream>

// QWidget
QWidget *QWidget_new(QWidget *parent) { return new QWidget{parent}; }

QLayout *QWidget_layout(QWidget *this_) { return this_->layout(); }

void QWidget_setLayout(QWidget *this_, QLayout *layout) {
  this_->setLayout(layout);
}

void QWidget_delete(QWidget *this_) {
  std::cerr << "QWidget_delete(" << this_ << ")\n";
  delete this_;
}

void QWidget_show(QWidget *this_) { this_->show(); }
