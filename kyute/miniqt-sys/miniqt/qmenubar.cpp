#include "callback.hpp"
#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QAction>
#include <QMenu>
#include <QMenuBar>

QMenuBar *QMenuBar_new(QWidget *parent) {
  return new QMenuBar(parent);
}

void QMenuBar_addMenu(QMenuBar *this_, QMenu *menu) {
  this_->addMenu(menu);
}

void QMenuBar_delete(QMenuBar *this_) { delete this_; }

QMenu *QMenu_new(QWidget *parent) { return new QMenu(parent); }

void QMenu_delete(QMenu *this_) { delete this_; }

void QMenu_setTitle(QMenu *this_, MQStringRef title) {
  this_->setTitle(makeQString(title));
}

QAction *QMenu_addAction(QMenu *this_, MQStringRef title) {
  return this_->addAction(makeQString(title));
}

QMenu *QMenu_addMenu(QMenu *this_, MQStringRef title) {
  return this_->addMenu(makeQString(title));
}

QAction *QMenu_addSeparator(QMenu *this_) {
  return this_->addSeparator();
}

/*
QAction *QMenu_addMenu(QMenu *this_, QMenu *menu) {
  return this_->addMenu(menu);
}
*/

/*
MQT_DEFINE_PTR_SIGNAL(QMenu, triggered, QAction)
MQT_DEFINE_VOID_SIGNAL(QAction, triggered)
*/