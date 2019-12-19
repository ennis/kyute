#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QVariant>
#include <QWidget>
#include <QLayout>
#include <iostream>

void QLayout_delete(QLayout *this_) { delete this_; }

QWidget *QLayout_widgetAt(QLayout *this_, int index) {
  std::cerr << "QLayout_widgetAt(" << this_ << "," << index << ")\n";
  auto layoutItem = this_->itemAt(index);
  std::cerr << "\tlayoutItem=" << layoutItem << "\n";
  if (!layoutItem)
    return nullptr;
  return layoutItem->widget();
}

int QLayout_count(QLayout *this_) { return this_->count(); }

int QLayout_indexOf_QWidget(QLayout *this_, QWidget *widget) {
  return this_->indexOf(widget);
}

QWidget *QLayout_replaceWidget(QLayout *this_, QWidget *from,
                                          QWidget *to) {
  std::cerr << "QLayout_replaceWidget(" << this_ << "," << from << "," << to
            << ")\n";
  auto item = this_->replaceWidget(from, to);
  return item->widget();
}

QLayoutItem *QLayout_takeAt(QLayout *this_, int i) {
	return this_->takeAt(i);
}

void QLayout_addItem(QLayout* this_, QLayoutItem* item) {
	this_->addItem(item);
}

void QLayout_addWidget(QLayout* this_, QWidget* widget) {
	this_->addWidget(widget);
}

QWidget* QLayoutItem_widget(QLayoutItem* this_) {
	return this_->widget();
}
