#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QBoxLayout>
#include <QHBoxLayout>
#include <QVBoxLayout>
#include <iostream>

/*
QLayout *QBoxLayout_upcast(QBoxLayout *this_) { return this_; }

void QBoxLayout_addWidget(QBoxLayout *this_, QWidget *widget) {
  std::cerr << "QBoxLayout_addWidget(" << this_ << "," << widget << ")\n";
  this_->addWidget(widget);
  std::cerr << "\tcount=" << this_->count() << "\n";
}

void QBoxLayout_delete(QBoxLayout *this_) {
	std::cerr << "QBoxLayout_delete(" << this_ << ")\n"; 
	delete this_; 
}

#define IMPL_BOX_LAYOUT(name)                                                  \
  Q##name##Layout *Q##name##Layout_new(QWidget *parent) {           \
    return new Q##name##Layout{parent};                                        \
  }                                                                            \
  QBoxLayout *Q##name##Layout_upcast(Q##name##Layout *this_) {      \
    return this_;                                                              \
  }                                                                            \
  void Q##name##Layout_delete(Q##name##Layout *this_) {             \
    delete this_;                                                              \
  }

IMPL_BOX_LAYOUT(HBox)
IMPL_BOX_LAYOUT(VBox)
*/