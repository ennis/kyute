#include "callback.hpp"
#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QLineEdit>

QLineEdit *QLineEdit_new(QWidget *parent, MQStringRef text) {
  return new QLineEdit(makeQString(text), parent);
}

QString *QLineEdit_text(QLineEdit *this_) {
  return new QString(this_->text());
}

void QLineEdit_setText(QLineEdit *this_, MQStringRef text) {
  this_->setText(makeQString(text));
}

void QLineEdit_delete(QLineEdit *this_) { delete this_; }

MQT_DEFINE_SIGNAL(QLineEdit, textChanged, QString)
MQT_DEFINE_SIGNAL(QLineEdit, textEdited, QString)
