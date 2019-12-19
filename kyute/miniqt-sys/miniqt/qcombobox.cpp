#include "callback.hpp"
#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QComboBox>

QComboBox *QComboBox_new(QWidget *parent) {
  return new QComboBox{parent};
}

QWidget *QComboBox_upcast(QComboBox *this_) { return this_; }

void
QComboBox_setSizeAjdustPolicy(QComboBox *this_,
                              QComboBox_SizeAdjustPolicy policy) {
  this_->setSizeAdjustPolicy(static_cast<QComboBox::SizeAdjustPolicy>(policy));
}

void QComboBox_setInsertPolicy(QComboBox *this_,
                                          QComboBox_InsertPolicy policy) {
  this_->setInsertPolicy(static_cast<QComboBox::InsertPolicy>(policy));
}

void QComboBox_insertItem(QComboBox *this_, int index,
                                     MQStringRef text, uintptr_t userData) {
  this_->insertItem(index, makeQString(text), userData);
}

void QComboBox_addItem(QComboBox *this_, MQStringRef text,
                                  uintptr_t userData) {
  this_->addItem(makeQString(text), userData);
}

void QComboBox_insertSeparator(QComboBox *this_, int index) {
  this_->insertSeparator(index);
}

void QComboBox_removeItem(QComboBox *this_, int index) {
  this_->removeItem(index);
}

int QComboBox_currentIndex(QComboBox *this_) {
  return this_->currentIndex();
}

void QComboBox_setCurrentIndex(QComboBox *this_, int index) {
  this_->setCurrentIndex(index);
}

void QComboBox_setEditText(QComboBox *this_, MQStringRef text) {
  this_->setEditText(makeQString(text));
}

void QComboBox_setEditable(QComboBox *this_, bool editable) {
  this_->setEditable(editable);
}

void QComboBox_delete(QComboBox *this_) { delete this_; }

MQCallback *QComboBox_activated_int(QComboBox *this_,
                                               uintptr_t data0, uintptr_t data1,
                                               MQCallbackFn_int callback) {
  auto cb = new miniqt::Callback_int(data0, data1, callback);
  this_->connect(this_, QOverload<int>::of(&QComboBox::activated), cb,
                 &miniqt::Callback_int::trigger);
  return cb;
}

MQCallback *
QComboBox_activated_QString(QComboBox *this_, uintptr_t data0, uintptr_t data1,
                            MQCallbackFn_QString callback) {
  auto cb = new miniqt::Callback_QString(data0, data1, callback);
  this_->connect(this_, QOverload<const QString &>::of(&QComboBox::activated),
                 cb, &miniqt::Callback_QString::trigger);
  return cb;
}

MQCallback *QComboBox_highlighted_int(QComboBox *this_,
                                                 uintptr_t data0,
                                                 uintptr_t data1,
                                                 MQCallbackFn_int callback) {
  auto cb = new miniqt::Callback_int(data0, data1, callback);
  this_->connect(this_, QOverload<int>::of(&QComboBox::highlighted), cb,
                 &miniqt::Callback_int::trigger);
  return cb;
}

MQCallback *
QComboBox_highlighted_QString(QComboBox *this_, uintptr_t data0,
                              uintptr_t data1, MQCallbackFn_QString callback) {
  auto cb = new miniqt::Callback_QString(data0, data1, callback);
  this_->connect(this_, QOverload<const QString &>::of(&QComboBox::highlighted),
                 cb, &miniqt::Callback_QString::trigger);
  return cb;
}

MQCallback *
QComboBox_currentIndexChanged_int(QComboBox *this_, uintptr_t data0,
                                  uintptr_t data1, MQCallbackFn_int callback) {
  auto cb = new miniqt::Callback_int(data0, data1, callback);
  this_->connect(this_, QOverload<int>::of(&QComboBox::currentIndexChanged), cb,
                 &miniqt::Callback_int::trigger);
  return cb;
}

MQT_DEFINE_SIGNAL(QComboBox, currentTextChanged, QString)
MQT_DEFINE_SIGNAL(QComboBox, editTextChanged, QString)
