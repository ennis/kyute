#include "callback.hpp"
#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QDebug>
#include <QEvent>
#include <QMetaEnum>
#include <QPushButton>
#include <iostream>

/*/// Gives human-readable event type information.
QDebug operator<<(QDebug str, const QEvent * ev) {
        static int eventEnumIndex = QEvent::staticMetaObject
                .indexOfEnumerator("Type");
        str << "QEvent";
        if (ev) {
                QString name = QEvent::staticMetaObject
                        .enumerator(eventEnumIndex).valueToKey(ev->type());
                if (!name.isEmpty()) str << name; else str << ev->type();
        }
        else {
                str << (void*)ev;
        }
        return str.maybeSpace();
}

class MQPushButton : public QPushButton {
public:
        MQPushButton(QWidget* parent = nullptr) : QPushButton{ parent }
        {}

protected:
        bool event(QEvent* e) override {
                // intercept the default event handler
                if (e->type() == QEvent::Paint) {
                        return QPushButton::event(e);
                }

                qDebug() << "MQPushButton: event intercepted " << e << "\n";
                return true;
        }
};*/

QPushButton *QPushButton_new(QWidget *parent) {
  return new QPushButton{parent};
}

QWidget *QPushButton_upcast(QPushButton *this_) { return this_; }

void QPushButton_setText(QPushButton *this_, MQStringRef text) {
  this_->setText(makeQString(text));
}

void QPushButton_delete(QPushButton *this_) {
  std::cerr << "QPushButton_delete(" << this_ << ")\n";
  delete this_;
}

MQT_DEFINE_VOID_SIGNAL(QPushButton, clicked)
MQT_DEFINE_VOID_SIGNAL(QPushButton, pressed)
MQT_DEFINE_VOID_SIGNAL(QPushButton, released)
MQT_DEFINE_VOID_SIGNAL(QPushButton, toggled)
