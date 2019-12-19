#include "miniqt.hpp"
#include "miniqthelpers.hpp"

#include <QApplication>
#include <QFont>
#include <QPalette>
#include <QStyleFactory>

/*
QApplication *QApplication_new(int *argc, char **argv) {
  auto d = new QApplication(*argc, argv);
  QApplication::setStyle(QStyleFactory::create("Fusion"));
  // modify palette to dark
  QPalette darkPalette;
  darkPalette.setColor(QPalette::Window, QColor(53, 53, 53));
  darkPalette.setColor(QPalette::WindowText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::WindowText,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::Base, QColor(42, 42, 42));
  darkPalette.setColor(QPalette::AlternateBase, QColor(66, 66, 66));
  darkPalette.setColor(QPalette::ToolTipBase, Qt::white);
  darkPalette.setColor(QPalette::ToolTipText, Qt::white);
  darkPalette.setColor(QPalette::Text, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::Text,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::Dark, QColor(35, 35, 35));
  darkPalette.setColor(QPalette::Shadow, QColor(20, 20, 20));
  darkPalette.setColor(QPalette::Button, QColor(53, 53, 53));
  darkPalette.setColor(QPalette::ButtonText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::ButtonText,
                       QColor(127, 127, 127));
  darkPalette.setColor(QPalette::BrightText, Qt::red);
  darkPalette.setColor(QPalette::Link, QColor(42, 130, 218));
  darkPalette.setColor(QPalette::Highlight, QColor(42, 130, 218));
  darkPalette.setColor(QPalette::Disabled, QPalette::Highlight,
                       QColor(80, 80, 80));
  darkPalette.setColor(QPalette::HighlightedText, Qt::white);
  darkPalette.setColor(QPalette::Disabled, QPalette::HighlightedText,
                       QColor(127, 127, 127));
  QApplication::setPalette(darkPalette);
  QFont font;
  font.setPointSizeF(11.0);
  QApplication::setFont(font);
  return d;
}

void QApplication_exec(QApplication *this_) { this_->exec(); }

void QApplication_setPalette(const QPalette *palette) {
  QApplication::setPalette(*palette);
}

void
QCoreApplication_processEvents(QEventLoop_ProcessEventsFlags flags) {
  QCoreApplication::processEvents(
      static_cast<QEventLoop::ProcessEventsFlags>(flags));
}
*/