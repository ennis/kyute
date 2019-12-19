#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QFormLayout>

 QFormLayout *QFormLayout_new() { return new QFormLayout; }

void QFormLayout_addRow_QWidget(QFormLayout *this_, QWidget *label,
                                           QWidget *field) {
  this_->addRow(label, field);
}

void QFormLayout_addRow_QString(QFormLayout *this_,
                                           MQStringRef labelText,
                                           QWidget *field) {
  this_->addRow(makeQString(labelText), field);
}

void QFormLayout_delete(QFormLayout *this_) { delete this_; }