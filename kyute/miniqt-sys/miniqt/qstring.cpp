#include "miniqt.hpp"
#include <QString>

const uint16_t *QString_utf16(const QString *this_) {
  return this_->utf16();
}

int QString_size(const QString *this_) { return this_->size(); }

void QString_delete(QString *this_) { delete this_; }