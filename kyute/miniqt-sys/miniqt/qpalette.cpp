#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QPalette>

QPalette *QPalette_new() { return new QPalette(); }

void QPalette_setColor(QPalette *this_, QPalette_ColorGroup group,
                       QPalette_ColorRole role, const double *color) {
  this_->setColor((QPalette::ColorGroup)group, (QPalette::ColorRole)role,
                  QColor::fromRgbF(color[0], color[1], color[2], color[3]));
}

void QPalette_setColor1(QPalette *this_, QPalette_ColorRole role,
                        const double *color) {
  this_->setColor((QPalette::ColorRole)role,
                  QColor::fromRgbF(color[0], color[1], color[2], color[3]));
}

void QPalette_delete(QPalette *this_) { delete this_; }
