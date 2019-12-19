#pragma once
#include "miniqt.hpp"

#include <QString>

/*
inline QString makeQString(MQStringRef stringRef) {
  if (stringRef.ptr == nullptr) {
    return QString();
  }
  return QString::fromUtf8(stringRef.ptr, (int)stringRef.len);
}

inline QByteArray makeQByteArray(MQStringRef stringRef) {
  if (stringRef.ptr == nullptr) {
    return QByteArray();
  }
  return QByteArray(stringRef.ptr, (int)stringRef.len);
}

inline MQString makeMQString(const QString& str) {
	auto byteArray = str.toUtf8();
	auto len = byteArray.size();
	MQString mqstr;
	mqstr.ptr = new char[len];
	mqstr.len = len;
	return mqstr;
}*/