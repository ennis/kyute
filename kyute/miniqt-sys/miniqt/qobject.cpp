#include "miniqt.hpp"
#include "miniqthelpers.hpp"
#include <QObject>
#include <QVariant>

bool QObject_setProperty_uint64(QObject *this_, MQStringRef name,
                                           uint64_t v) {
  return this_->setProperty(makeQByteArray(name), (quint64)v);
}

uint64_t QObject_property_uint64(QObject *this_, MQStringRef name) {
  QVariant v = this_->property(makeQByteArray(name));
  return v.value<quint64>();
}